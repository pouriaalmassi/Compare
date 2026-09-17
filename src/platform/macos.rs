use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::Stream;
use iced::Subscription;
use objc::runtime::{
    class_addMethod, class_getInstanceMethod, method_setImplementation, Class, Imp, Method,
    Object, Sel, BOOL, YES,
};
use objc::{msg_send, sel, sel_impl};
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::app::Message;

static SUBSCRIBERS: Mutex<Vec<UnboundedSender<Vec<PathBuf>>>> = Mutex::new(Vec::new());
static PENDING_BATCHES: Mutex<Vec<Vec<PathBuf>>> = Mutex::new(Vec::new());
static mut ORIGINAL_SET_DELEGATE: Option<unsafe extern "C" fn(*mut Object, Sel, *mut Object)> =
    None;

pub fn send_paths(paths: Vec<PathBuf>) {
    if paths.is_empty() {
        return;
    }
    let mut subs = SUBSCRIBERS.lock().unwrap();
    if subs.is_empty() {
        PENDING_BATCHES.lock().unwrap().push(paths);
    } else {
        subs.retain_mut(|tx| tx.unbounded_send(paths.clone()).is_ok());
    }
}

fn subscribe() -> impl Stream<Item = Message> {
    use futures::StreamExt;
    let (tx, rx) = unbounded();
    {
        let mut pending = PENDING_BATCHES.lock().unwrap();
        for batch in pending.drain(..) {
            let _ = tx.unbounded_send(batch);
        }
        let mut subs = SUBSCRIBERS.lock().unwrap();
        subs.push(tx);
    }
    rx.map(Message::DockFilesDropped)
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(subscribe)
}

unsafe fn reply_success(app: *mut Object) {
    let reply_sel = sel!(replyToOpenOrPrint:);
    let target = if !app.is_null() {
        app
    } else if let Some(cls) = Class::get("NSApplication") {
        let shared: *mut Object = msg_send![cls, sharedApplication];
        shared
    } else {
        std::ptr::null_mut()
    };

    if !target.is_null() {
        let responds: BOOL = msg_send![target, respondsToSelector: reply_sel];
        if responds == YES {
            let _: () = msg_send![target, replyToOpenOrPrint: 0usize];
        }
    }
}

unsafe extern "C" fn open_files_imp(
    _this: *mut Object,
    _cmd: Sel,
    app: *mut Object,
    filenames: *mut Object,
) {
    if !filenames.is_null() {
        let count: usize = msg_send![filenames, count];
        let mut paths = Vec::with_capacity(count);
        for i in 0..count {
            let item: *mut Object = msg_send![filenames, objectAtIndex: i];
            if !item.is_null() {
                let utf8_ptr: *const c_char = msg_send![item, UTF8String];
                if !utf8_ptr.is_null() {
                    if let Ok(s) = CStr::from_ptr(utf8_ptr).to_str() {
                        paths.push(PathBuf::from(s));
                    }
                }
            }
        }
        if !paths.is_empty() {
            send_paths(paths);
        }
    }

    reply_success(app);
}

unsafe extern "C" fn open_urls_imp(
    _this: *mut Object,
    _cmd: Sel,
    app: *mut Object,
    urls: *mut Object,
) {
    if !urls.is_null() {
        let count: usize = msg_send![urls, count];
        let mut paths = Vec::with_capacity(count);
        for i in 0..count {
            let url: *mut Object = msg_send![urls, objectAtIndex: i];
            if !url.is_null() {
                let path_obj: *mut Object = msg_send![url, path];
                if !path_obj.is_null() {
                    let utf8_ptr: *const c_char = msg_send![path_obj, UTF8String];
                    if !utf8_ptr.is_null() {
                        if let Ok(s) = CStr::from_ptr(utf8_ptr).to_str() {
                            paths.push(PathBuf::from(s));
                        }
                    }
                }
            }
        }
        if !paths.is_empty() {
            send_paths(paths);
        }
    }

    reply_success(app);
}

unsafe extern "C" fn open_file_imp(
    _this: *mut Object,
    _cmd: Sel,
    _app: *mut Object,
    filename: *mut Object,
) -> BOOL {
    if !filename.is_null() {
        let utf8_ptr: *const c_char = msg_send![filename, UTF8String];
        if !utf8_ptr.is_null() {
            if let Ok(s) = CStr::from_ptr(utf8_ptr).to_str() {
                send_paths(vec![PathBuf::from(s)]);
            }
        }
    }
    YES
}

unsafe fn add_open_file_methods(cls: *mut Class) {
    if cls.is_null() {
        return;
    }
    let types = CString::new("v@:@@").unwrap();
    let types_bool = CString::new("c@:@@").unwrap();

    class_addMethod(
        cls,
        sel!(application:openFiles:),
        std::mem::transmute::<
            unsafe extern "C" fn(*mut Object, Sel, *mut Object, *mut Object),
            Imp,
        >(open_files_imp),
        types.as_ptr(),
    );

    class_addMethod(
        cls,
        sel!(application:openURLs:),
        std::mem::transmute::<
            unsafe extern "C" fn(*mut Object, Sel, *mut Object, *mut Object),
            Imp,
        >(open_urls_imp),
        types.as_ptr(),
    );

    class_addMethod(
        cls,
        sel!(application:openFile:),
        std::mem::transmute::<
            unsafe extern "C" fn(*mut Object, Sel, *mut Object, *mut Object) -> BOOL,
            Imp,
        >(open_file_imp),
        types_bool.as_ptr(),
    );
}

unsafe extern "C" fn swizzled_set_delegate(this: *mut Object, sel: Sel, delegate: *mut Object) {
    if !delegate.is_null() {
        let cls: *const Class = msg_send![delegate, class];
        add_open_file_methods(cls as *mut Class);
    }
    if let Some(orig) = ORIGINAL_SET_DELEGATE {
        orig(this, sel, delegate);
    }
}

pub fn init() {
    unsafe {
        if let Some(cls) = Class::get("NSObject") {
            add_open_file_methods(cls as *const Class as *mut Class);
        }

        if let Some(cls) = Class::get("WinitApplicationDelegate") {
            add_open_file_methods(cls as *const Class as *mut Class);
        }

        if let Some(cls) = Class::get("NSApplication") {
            let method = class_getInstanceMethod(cls, sel!(setDelegate:));
            if !method.is_null() {
                let orig_imp = method_setImplementation(
                    method as *mut Method,
                    std::mem::transmute::<
                        unsafe extern "C" fn(*mut Object, Sel, *mut Object),
                        Imp,
                    >(swizzled_set_delegate),
                );
                ORIGINAL_SET_DELEGATE = Some(std::mem::transmute::<
                    Imp,
                    unsafe extern "C" fn(*mut Object, Sel, *mut Object),
                >(orig_imp));
            }
        }
    }
}
