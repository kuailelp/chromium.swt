extern crate chromium;

#[cfg(target_os = "linux")]
extern crate x11;
#[cfg(unix)]
extern crate nix;
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use chromium::cef;
use chromium::utils;

mod app;
#[cfg(target_os = "linux")]
mod gtk2;

use std::os::raw::{c_char, c_int, c_ulong, c_void};
#[cfg(unix)]
use std::collections::HashMap;

#[cfg(target_os = "linux")]
unsafe extern fn xerror_handler_impl(_: *mut x11::xlib::Display, event: *mut x11::xlib::XErrorEvent) -> c_int {
    print!("X error received: ");
    println!("type {}, serial {}, error_code {}, request_code {}, minor_code {}", 
        (*event).type_, (*event).serial, (*event).error_code, (*event).request_code, (*event).minor_code);
    0
}
#[cfg(target_os = "linux")]
unsafe extern fn xioerror_handler_impl(_: *mut x11::xlib::Display) -> c_int {
    println!("XUI error received");
    0
}

#[no_mangle]
pub extern fn cefswt_init(japp: *mut cef::cef_app_t, cefrust_path: *const c_char, version: *const c_char) {
    println!("DLL init");
    assert_eq!(unsafe{(*japp).base.size}, std::mem::size_of::<cef::_cef_app_t>());
    //println!("app {:?}", japp);

    let cefrust_path = utils::str_from_c(cefrust_path);
    let version = utils::str_from_c(version);

    // let key = "LD_LIBRARY_PATH";
    // env::set_var(key, cefrust_path);

    let main_args = utils::prepare_args();

    let cefrust_dir = std::path::Path::new(&cefrust_path);

    // env::set_current_dir(cefrust_dir).expect("Failed to set current dir");
    // println!("{:?}", env::current_dir().unwrap().to_str());

    let subp = utils::subp_path(cefrust_dir, version);
    let subp_cef = utils::cef_string(&subp);
    
    let resources_cef = if cfg!(target_os = "macos") {
        utils::cef_string(cefrust_dir.join("Chromium Embedded Framework.framework").join("Resources").to_str().unwrap())
    } else {
        utils::cef_string(cefrust_dir.to_str().unwrap())
    };
    let locales_cef = if cfg!(target_os = "macos") {
        utils::cef_string(cefrust_dir.join("Chromium Embedded Framework.framework").join("Resources").to_str().unwrap())
    } else {
        utils::cef_string(cefrust_dir.join("locales").to_str().unwrap())
    };
    let framework_dir_cef = if cfg!(target_os = "macos") {
        utils::cef_string(cefrust_dir.join("Chromium Embedded Framework.framework").to_str().unwrap())
    } else {
        utils::cef_string_empty()
    };

    let cache_dir_cef = utils::cef_string(cefrust_dir.parent().unwrap().parent().unwrap().join("cef_cache").to_str().unwrap());

    let logfile_cef = utils::cef_string(cefrust_dir.join("lib.log").to_str().unwrap());

    let settings = cef::_cef_settings_t {
        size: std::mem::size_of::<cef::_cef_settings_t>(),
        single_process: 0,
        no_sandbox: 1,
        browser_subprocess_path: subp_cef,
        framework_dir_path: framework_dir_cef,
        multi_threaded_message_loop: 0,
        external_message_pump: 1,
        windowless_rendering_enabled: 0,
        command_line_args_disabled: 0,
        cache_path: cache_dir_cef,
        user_data_path: utils::cef_string_empty(),
        persist_session_cookies: 1,
        persist_user_preferences: 1,
        user_agent: utils::cef_string_empty(),
        product_version: utils::cef_string_empty(),
        locale: utils::cef_string_empty(),
        log_file: logfile_cef,
        log_severity: cef::cef_log_severity_t::LOGSEVERITY_INFO,
        //log_severity: cef::cef_log_severity_t::LOGSEVERITY_VERBOSE,
        javascript_flags: utils::cef_string_empty(),
        resources_dir_path: resources_cef,
        locales_dir_path: locales_cef,
        pack_loading_disabled: 0,
        remote_debugging_port: 0,
        uncaught_exception_stack_size: 0,
        ignore_certificate_errors: 0,
        enable_net_security_expiration: 0,
        background_color: 0,
        accept_language_list: utils::cef_string_empty()
    };

    println!("Calling cef_initialize");
    do_initialize(main_args, settings, japp);
}

#[cfg(target_os = "linux")]
fn do_initialize(main_args: cef::_cef_main_args_t, settings: cef::_cef_settings_t, app_raw: *mut cef::_cef_app_t) {
    unsafe { x11::xlib::XSetErrorHandler(Option::Some(xerror_handler_impl)) };
    unsafe { x11::xlib::XSetIOErrorHandler(Option::Some(xioerror_handler_impl)) };

    let mut signal_handlers: HashMap<c_int, nix::sys::signal::SigAction> = HashMap::new();
    backup_signal_handlers(&mut signal_handlers);
    
    unsafe { cef::cef_initialize(&main_args, &settings, app_raw, std::ptr::null_mut()) };

    restore_signal_handlers(signal_handlers);
}

#[cfg(target_os = "macos")]
static EVENT_KEY: char = 'k';

#[cfg(target_os = "macos")]
fn do_initialize(main_args: cef::_cef_main_args_t, settings: cef::_cef_settings_t, app_raw: *mut cef::_cef_app_t) {
    let mut signal_handlers: HashMap<c_int, nix::sys::signal::SigAction> = HashMap::new();
    backup_signal_handlers(&mut signal_handlers);
    
    swizzle_send_event();

    unsafe { cef::cef_initialize(&main_args, &settings, &mut (*app_raw), std::ptr::null_mut()) };

    restore_signal_handlers(signal_handlers);
}

#[cfg(target_os = "macos")]
fn swizzle_send_event() {
    use std::ffi::CString;
    use objc::runtime::{BOOL, Class, Method, NO, YES, Object, Sel, self};
    use objc::{Encode, EncodeArguments, Encoding};
    use nix::libc::intptr_t;

    fn count_args(sel: Sel) -> usize {
        sel.name().chars().filter(|&c| c == ':').count()
    }

    fn method_type_encoding(ret: &Encoding, args: &[Encoding]) -> CString {
        let mut types = ret.as_str().to_owned();
        // First two arguments are always self and the selector
        types.push_str(<*mut Object>::encode().as_str());
        types.push_str(Sel::encode().as_str());
        types.extend(args.iter().map(|e| e.as_str()));
        CString::new(types).unwrap()
    }
    
    pub unsafe fn add_method<F>(cls: *mut Class, sel: Sel, func: F)
            where F: objc::declare::MethodImplementation<Callee=Object> {
        let encs = F::Args::encodings();
        let encs = encs.as_ref();
        let sel_args = count_args(sel);
        assert!(sel_args == encs.len(),
            "Selector accepts {} arguments, but function accepts {}",
            sel_args, encs.len(),
        );

        let types = method_type_encoding(&F::Ret::encode(), encs);
        let success = runtime::class_addMethod(cls, sel, func.imp(),
            types.as_ptr());
        assert!(success != NO, "Failed to add method {:?}", sel);
    }

    pub type Id = *mut runtime::Object;
    pub type AssociationPolicy = intptr_t;
    extern {
        pub fn objc_getAssociatedObject(object: Id, key: *const c_void) -> BOOL;
        pub fn objc_setAssociatedObject(object: Id,
                                    key: *const c_void,
                                    value: BOOL,
                                    policy: AssociationPolicy);
    }

    let cls_nm = CString::new("NSApplication").unwrap();
    let cls = unsafe { runtime::objc_getClass(cls_nm.as_ptr()) as *mut Class };
    assert!(!cls.is_null(), "null class");

    extern fn is_handling_sendevent(this: &mut Object, _cmd: Sel) -> BOOL {
        //println!("isHandlingSendEvent {:?}", this);
        let kp = &EVENT_KEY as *const _ as *const c_void;
        let is = unsafe { objc_getAssociatedObject(this, kp) };
        //println!("AssociatedObject: {:?}", is);
        is
    }
    unsafe { add_method(cls, sel!(isHandlingSendEvent), is_handling_sendevent as extern fn(&mut Object, Sel) -> BOOL) };

    extern fn set_handling_sendevent(this: &mut Object, _cmd: Sel, handling_sendevent: BOOL) {
        //println!("setHandlingSendEvent {:?} {:?}", this, handling_sendevent);
        let kp = &EVENT_KEY as *const _ as *const c_void;
        let policy_assign = 0;
        unsafe { objc_setAssociatedObject(this, kp, handling_sendevent, policy_assign) };
    }
    unsafe { add_method(cls, sel!(setHandlingSendEvent:), set_handling_sendevent as extern fn(&mut Object, Sel, BOOL)) };

    extern fn swizzled_sendevent(this: &mut Object, _cmd: Sel, event: Id) {
        //println!("swizzled_sendevent {:?}", this);
        unsafe {
            let handling: BOOL = msg_send![this, isHandlingSendEvent];
            msg_send![this, setHandlingSendEvent:YES];
            msg_send![this, _swizzled_sendEvent:event];
            msg_send![this, setHandlingSendEvent:handling];
        }
    }
    let sel_swizzled_sendevent = sel!(_swizzled_sendEvent:);
    unsafe { add_method(cls, sel_swizzled_sendevent, swizzled_sendevent as extern fn(&mut Object, Sel, Id)) };
    
    unsafe {
        let original = runtime::class_getInstanceMethod(cls, sel!(sendEvent:)) as *mut Method;
        let swizzled = runtime::class_getInstanceMethod(cls, sel_swizzled_sendevent) as *mut Method;
        runtime::method_exchangeImplementations(original, swizzled);
    }
}

#[cfg(target_os = "windows")]
fn do_initialize(main_args: cef::_cef_main_args_t, settings: cef::_cef_settings_t, app_raw: *mut cef::_cef_app_t) {
    unsafe { cef::cef_initialize(&main_args, &settings, &mut (*app_raw), std::ptr::null_mut()) };
}

#[cfg(unix)]
fn backup_signal_handlers(signal_handlers: &mut HashMap<c_int, nix::sys::signal::SigAction>) {
    use nix::sys::signal;
    let signals_to_restore = [signal::SIGHUP, signal::SIGINT, signal::SIGQUIT, signal::SIGILL, 
        signal::SIGABRT, signal::SIGFPE, signal::SIGSEGV, signal::SIGALRM, signal::SIGTERM, 
        signal::SIGCHLD, signal::SIGBUS, signal::SIGTRAP, signal::SIGPIPE];
    
    for signal in &signals_to_restore {
        let sig_action = signal::SigAction::new(signal::SigHandler::SigDfl,
                                          signal::SaFlags::empty(),
                                          signal::SigSet::empty());
        let oldsigact = unsafe { signal::sigaction(*signal, &sig_action) };
        //println!("backup signal {:?}:{:?}", signal, "oldsigact.ok()");
        signal_handlers.insert(*signal as c_int, oldsigact.unwrap());
    }
}

#[cfg(unix)]
fn restore_signal_handlers(signal_handlers: HashMap<c_int, nix::sys::signal::SigAction>) {
    use nix::sys::signal;
    for (signal, sigact) in signal_handlers {
        //println!("restore signal {:?}:{:?}", signal, "sigact");
        unsafe { signal::sigaction(std::mem::transmute(signal), &sigact).unwrap() };
    }
}

#[no_mangle]
pub extern fn cefswt_create_browser(hwnd: c_ulong, url: *const c_char, client: &mut cef::_cef_client_t, w: c_int, h: c_int) -> *const cef::cef_browser_t {
    println!("create_browser");
    assert_eq!((*client).base.size, std::mem::size_of::<cef::_cef_client_t>());

    // println!("hwnd: {}", hwnd);
 
    let url = utils::str_from_c(url);
    // println!("url: {:?}", url);
    let browser = app::create_browser(hwnd, url, client, w, h);

    browser
}

#[no_mangle]
pub extern fn cefswt_do_message_loop_work() {
    unsafe { cef::cef_do_message_loop_work() };
}

#[no_mangle]
pub extern fn cefswt_free(obj: *mut cef::cef_browser_t) {
    //println!("freeing {:?}", obj);
    unsafe {
        assert_eq!((*obj).base.size, std::mem::size_of::<cef::_cef_browser_t>());

        let rls_fn = (*obj).base.release.expect("null release");
        // println!("call rls");
        let refs = rls_fn(obj as *mut cef::_cef_base_ref_counted_t);
        assert_eq!(refs, 1);
    }

    println!("freed");
}

#[no_mangle]
pub extern fn cefswt_resized(browser: *mut cef::cef_browser_t, width: i32, height: i32) {
    //println!("Calling resized {}:{}", width, height);
    
    let browser_host = get_browser_host(browser);
    let get_window_handle_fn = unsafe { (*browser_host).get_window_handle.expect("no get_window_handle") };
    let win_handle = unsafe { get_window_handle_fn(browser_host) };
    do_resize(win_handle, width, height);
}

#[cfg(target_os = "linux")]
fn do_resize(win_handle: c_ulong, width: i32, height: i32) {
    use x11::xlib;

    let xwindow = win_handle;
    let xdisplay = unsafe { cef::cef_get_xdisplay() };
    let mut changes = xlib::XWindowChanges {
        x: 0,
        y: 0,
        width: width,
        height: height,
        border_width: 0,
        sibling: 0,
        stack_mode: 0
    };
    unsafe { xlib::XConfigureWindow(std::mem::transmute(xdisplay), xwindow,
        (xlib::CWX | xlib::CWY | xlib::CWHeight | xlib::CWWidth) as u32, &mut changes) };
}

#[cfg(target_os = "macos")]
fn do_resize(_win_handle: c_ulong, _: i32, _: i32) {
    // handled by cocoa
}

#[cfg(target_family = "windows")]
fn do_resize(win_handle: c_ulong, width: i32, height: i32) {
    extern crate winapi;

    let x = 0;
    let y = 0;
    unsafe { winapi::um::winuser::SetWindowPos(win_handle as winapi::shared::windef::HWND, 
        std::ptr::null_mut(), x, y, width, height, winapi::um::winuser::SWP_NOZORDER) };
}

#[no_mangle]
pub extern fn cefswt_close_browser(browser: *mut cef::cef_browser_t) {
    let browser_host = get_browser_host(browser);
    let close_fn = unsafe { (*browser_host).close_browser.expect("null try_close_browser") };
    unsafe { close_fn(browser_host, 1) };
}

#[no_mangle]
pub extern fn cefswt_load_url(browser: *mut cef::cef_browser_t, url: *const c_char) {
    let url = utils::str_from_c(url);
    let url_cef = utils::cef_string(url);
    println!("url: {:?}", url);
    let get_frame = unsafe { (*browser).get_main_frame.expect("null get_main_frame") };
    let main_frame = unsafe { get_frame(browser) };
    let load_url = unsafe { (*main_frame).load_url.expect("null load_url") };
    unsafe { load_url(main_frame, &url_cef) };
}

#[no_mangle]
pub extern fn cefswt_get_url(browser: *mut cef::cef_browser_t) -> *mut c_char {
    let get_frame = unsafe { (*browser).get_main_frame.expect("null get_main_frame") };
    let main_frame = unsafe { get_frame(browser) };
    assert!(!main_frame.is_null());
    let get_url = unsafe { (*main_frame).get_url.expect("null get_url") };
    let url = unsafe { get_url(main_frame) };
    if url.is_null() {
        return std::ptr::null_mut();
    } else {
        let utf8 = unsafe { cef::cef_string_userfree_utf8_alloc()};
        unsafe { cef::cef_string_utf16_to_utf8((*url).str, (*url).length, utf8) };
        return unsafe {(*utf8).str};
    }
}

#[no_mangle]
pub extern fn cefswt_set_focus(browser: *mut cef::cef_browser_t, set: bool, parent: *mut c_void) {
    let browser_host = get_browser_host(browser);
    let focus_fn = unsafe { (*browser_host).set_focus.expect("null set_focus") };
    let focus = if set {
        1
    } else {
        0
    };
    println!("<<<<<<<< set_focus {}", focus);
    unsafe { focus_fn(browser_host, focus) };
    if !set && parent as c_ulong != 0 {
        do_set_focus(parent, focus);
    }
}

#[cfg(target_os = "linux")]
fn do_set_focus(parent: *mut c_void, focus: i32) {
    let root = unsafe { gtk2::gtk_widget_get_toplevel(parent) };
    println!("<<<<<<<< set_focus {} {:?} {:?}", focus, parent, root);
    // workaround to actually remove focus from cef inputs
    unsafe { gtk2::gtk_window_present(root) };
}

#[cfg(target_family = "windows")]
fn do_set_focus(_parent: *mut c_void, _focus: i32) {
    // TODO
}

#[cfg(target_os = "macos")]
fn do_set_focus(_parent: *mut c_void, _focus: i32) {
    // handled by cocoa
}

#[no_mangle]
pub extern fn cefswt_shutdown() {
    println!("r: Calling cef_shutdown");
    // Shut down CEF.
    unsafe { cef::cef_shutdown() };
    // println!("r: After Calling cef_shutdown");
}

fn get_browser_host(browser: *mut cef::cef_browser_t) -> *mut cef::_cef_browser_host_t {
    let get_host_fn = unsafe { (*browser).get_host.expect("null get_host") };
    let browser_host = unsafe { get_host_fn(browser) };
    browser_host
}
