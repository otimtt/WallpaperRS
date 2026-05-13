use anyhow::Result;
use wallpaper_core::renderer::Renderer;

// ── Windows — WebView2 child-window embedding ─────────────────────────────────

#[cfg(target_os = "windows")]
mod wv2 {
    use anyhow::{anyhow, Result};
    use std::time::{Duration, Instant};
    use windows::{
        Win32::{
            Foundation::{HWND, RECT, S_FALSE},
            System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            UI::WindowsAndMessaging::{
                DispatchMessageW, MSG, PeekMessageW, TranslateMessage, PM_REMOVE,
            },
        },
        core::HRESULT,
    };
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        CreateCoreWebView2Environment,
        ICoreWebView2,
        ICoreWebView2Controller,
        ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
        ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl,
        ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
        ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl,
        ICoreWebView2Environment,
    };
    use windows::core::implement;

    // ── COM callback helpers ──────────────────────────────────────────────────

    // We pass results back via raw pointer because ICoreWebView2* are !Send.
    // Safety: all callbacks are dispatched on the same STA thread via pump_until.

    struct EnvResultPtr(*mut Option<windows::core::Result<ICoreWebView2Environment>>);
    unsafe impl Send for EnvResultPtr {}
    unsafe impl Sync for EnvResultPtr {}

    #[implement(ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler)]
    struct EnvHandler(EnvResultPtr);

    impl ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl for EnvHandler_Impl {
        fn Invoke(
            &self,
            error_code: HRESULT,
            created_environment: Option<&ICoreWebView2Environment>,
        ) -> windows::core::Result<()> {
            let result = if error_code.is_ok() {
                created_environment
                    .cloned()
                    .ok_or_else(|| windows::core::Error::from(error_code))
            } else {
                Err(windows::core::Error::from(error_code))
            };
            unsafe { *(self.0).0 = Some(result); }
            Ok(())
        }
    }

    struct CtrlResultPtr(*mut Option<windows::core::Result<ICoreWebView2Controller>>);
    unsafe impl Send for CtrlResultPtr {}
    unsafe impl Sync for CtrlResultPtr {}

    #[implement(ICoreWebView2CreateCoreWebView2ControllerCompletedHandler)]
    struct CtrlHandler(CtrlResultPtr);

    impl ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl for CtrlHandler_Impl {
        fn Invoke(
            &self,
            error_code: HRESULT,
            created_controller: Option<&ICoreWebView2Controller>,
        ) -> windows::core::Result<()> {
            let result = if error_code.is_ok() {
                created_controller
                    .cloned()
                    .ok_or_else(|| windows::core::Error::from(error_code))
            } else {
                Err(windows::core::Error::from(error_code))
            };
            unsafe { *(self.0).0 = Some(result); }
            Ok(())
        }
    }

    // ── Message pump ─────────────────────────────────────────────────────────

    fn pump_until(ready: impl Fn() -> bool, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if ready() { return true; }
            if Instant::now() >= deadline { return false; }
            unsafe {
                let mut msg = MSG::default();
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }

    // ── Wv2Embed ──────────────────────────────────────────────────────────────

    pub struct Wv2Embed {
        pub width:  u32,
        pub height: u32,
        controller:      ICoreWebView2Controller,
        #[allow(dead_code)]
        webview:         ICoreWebView2,
        com_initialized: bool,
    }

    impl Wv2Embed {
        pub fn new(parent_hwnd: usize, url: &str, w: u32, h: u32) -> Result<Self> {
            // Initialize COM on this thread (STA required for WebView2)
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            // S_OK = initialized; S_FALSE = already initialized on this thread (both fine)
            let com_initialized = hr != S_FALSE;
            if !hr.is_ok() && hr != S_FALSE {
                return Err(anyhow!("CoInitializeEx failed: {:?}", hr));
            }

            let env = create_env()?;
            let hwnd = HWND(parent_hwnd as isize);
            let (controller, webview) = create_controller(&env, hwnd, w, h)?;

            // Navigate to URL
            let url_wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                webview.Navigate(windows::core::PCWSTR(url_wide.as_ptr()))
                    .unwrap_or_else(|e| log::warn!("WebView2 navigate failed: {e}"));
            }

            log::info!("WebView2 initialized → {url}");
            Ok(Self { width: w, height: h, controller, webview, com_initialized })
        }

        pub fn resize(&mut self, w: u32, h: u32) {
            self.width = w; self.height = h;
            let bounds = RECT { left: 0, top: 0, right: w as i32, bottom: h as i32 };
            unsafe {
                let _ = self.controller.SetBounds(bounds);
            }
        }
    }

    impl Drop for Wv2Embed {
        fn drop(&mut self) {
            if self.com_initialized {
                unsafe { CoUninitialize(); }
            }
        }
    }

    // Safety: Wv2Embed is only ever used from the render thread (STA).
    unsafe impl Send for Wv2Embed {}
    unsafe impl Sync for Wv2Embed {}

    // ── Async env/controller helpers ─────────────────────────────────────────

    fn create_env() -> Result<ICoreWebView2Environment> {
        let mut slot: Option<windows::core::Result<ICoreWebView2Environment>> = None;
        let ptr: *mut _ = &mut slot;
        let handler: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler =
            EnvHandler(EnvResultPtr(ptr)).into();

        unsafe { CreateCoreWebView2Environment(&handler)? };

        if !pump_until(|| unsafe { (*ptr).is_some() }, Duration::from_secs(30)) {
            return Err(anyhow!("WebView2 environment creation timed out (30 s)"));
        }

        slot.take()
            .unwrap()
            .map_err(|e| anyhow!("WebView2 environment error: {e}"))
    }

    fn create_controller(
        env: &ICoreWebView2Environment,
        hwnd: HWND,
        w: u32,
        h: u32,
    ) -> Result<(ICoreWebView2Controller, ICoreWebView2)> {
        let mut slot: Option<windows::core::Result<ICoreWebView2Controller>> = None;
        let ptr: *mut _ = &mut slot;
        let handler: ICoreWebView2CreateCoreWebView2ControllerCompletedHandler =
            CtrlHandler(CtrlResultPtr(ptr)).into();

        unsafe { env.CreateCoreWebView2Controller(hwnd, &handler)? };

        if !pump_until(|| unsafe { (*ptr).is_some() }, Duration::from_secs(30)) {
            return Err(anyhow!("WebView2 controller creation timed out (30 s)"));
        }

        let controller = slot
            .take()
            .unwrap()
            .map_err(|e| anyhow!("WebView2 controller error: {e}"))?;

        // Cover the entire window
        let bounds = RECT { left: 0, top: 0, right: w as i32, bottom: h as i32 };
        unsafe {
            controller.SetBounds(bounds)?;
            controller.put_IsVisible(true)?;
        }

        let webview = unsafe { controller.get_CoreWebView2()? };
        Ok((controller, webview))
    }
}

// ── WebRenderer ───────────────────────────────────────────────────────────────

pub struct WebRenderer {
    width:  u32,
    height: u32,
    #[allow(dead_code)]
    url: String,
    #[cfg(target_os = "windows")]
    wv2: Option<wv2::Wv2Embed>,
}

impl WebRenderer {
    pub fn new(
        parent_hwnd: usize,
        url: String,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        #[cfg(target_os = "windows")]
        let wv2 = match wv2::Wv2Embed::new(parent_hwnd, &url, width, height) {
            Ok(w)  => { log::info!("WebView2 ready"); Some(w) }
            Err(e) => { log::warn!("WebView2 unavailable: {e}"); None }
        };

        Ok(Self {
            width,
            height,
            url,
            #[cfg(target_os = "windows")]
            wv2,
        })
    }
}

impl Renderer for WebRenderer {
    fn update(&mut self, _delta: f32) {}

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) -> Result<()> {
        // WebView2 renders in its child window on top of the WGPU surface.
        // We only need to clear the surface; when WebView2 is unavailable we show a fallback colour.
        let clear = {
            #[cfg(target_os = "windows")]
            {
                if self.wv2.is_some() {
                    wgpu::Color::BLACK
                } else {
                    wgpu::Color { r: 0.05, g: 0.0, b: 0.02, a: 1.0 }
                }
            }
            #[cfg(not(target_os = "windows"))]
            wgpu::Color { r: 0.05, g: 0.0, b: 0.02, a: 1.0 }
        };

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Web Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.width  = w;
        self.height = h;
        #[cfg(target_os = "windows")]
        if let Some(wv2) = &mut self.wv2 {
            wv2.resize(w, h);
        }
    }

    fn name(&self) -> &str { "Web" }
}
