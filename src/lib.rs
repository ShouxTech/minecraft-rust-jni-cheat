use jni::{JavaVM, JNIEnv};
use jni::objects::{JValue, JObject};
use jni::sys::{jsize, jint};
use windows::Win32::System::Console::AllocConsole;
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, s};
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, core::*, Win32::System::LibraryLoader::* };

enum JVMError {
    DllNotFound,
    JvmNotFound,
}

impl ToString for JVMError {
    fn to_string(&self) -> String {
        match self {
            JVMError::DllNotFound => "DllNotFound".to_owned(),
            JVMError::JvmNotFound => "JvmNotFound".to_owned(),
        }
    }
}
trait UnwrapOrMsgBoxAndExit<T, E: std::fmt::Display> {
    fn unwrap_or_msgbox_and_exit(self, text: Option<&str>) -> T;
}

impl<T, E: std::fmt::Display> UnwrapOrMsgBoxAndExit<T, E> for std::result::Result<T, E> {
    fn unwrap_or_msgbox_and_exit(self, text: Option<&str>) -> T {
        return self.unwrap_or_else(|err| {
            if let Some(t) = text {
                message_box(t);
            } else {
                message_box(err.to_string().as_str());
            }
            std::process::exit(1);
        });
    }
}

type JNIGetCreatedJavaVMs = fn (vmBuf: *mut *mut JavaVM, bufLen: jsize, nVMs: *mut jsize) -> jint;

extern "C" {
    // These actually return a *mut FILE but using *mut isize for simplicity.
    fn __acrt_iob_func(index: u32) -> *mut isize;
    fn freopen(filename: PCSTR, mode: PCSTR, stream: *mut isize) -> *mut isize;
}

fn message_box(text: &str) {
    unsafe {
        let null_terminated_text = text.to_owned() + "\0";

        MessageBoxA(HWND(0), PCSTR(null_terminated_text.as_ptr()), s!("dll"), Default::default());
    }
}

fn create_console() {
    unsafe {
        AllocConsole();
        freopen(s!("CONIN$"), s!("r"), __acrt_iob_func(0)); // stdin
        freopen(s!("CONOUT$"), s!("w"), __acrt_iob_func(1)); // stdout
        freopen(s!("CONOUT$"), s!("w"), __acrt_iob_func(2)); // stderr
    }
}

// Unusued for now.
fn _get_base_address(module_name: Option<PCSTR>) -> isize {
    unsafe {
        let handle = match module_name {
            Some(s) => GetModuleHandleA(s),
            None => GetModuleHandleA(None),
        }.unwrap();
        return handle.0;
    }
}

fn get_jvm() -> std::result::Result<JavaVM, JVMError> {
    unsafe {
        let jvm_dll = match GetModuleHandleA(s!("jvm.dll")) {
            Ok(jvm_dll) => jvm_dll,
            Err(_) => return Err(JVMError::DllNotFound),
        };

        let jni_get_created_java_vms_ptr = GetProcAddress(jvm_dll, s!("JNI_GetCreatedJavaVMs")).unwrap();

        let jni_get_created_java_vms: JNIGetCreatedJavaVMs = std::mem::transmute(jni_get_created_java_vms_ptr);

        let mut jvm_ptr: *mut JavaVM = std::ptr::null_mut();
        let mut found: jsize = 0;

        jni_get_created_java_vms(&mut jvm_ptr, 1, &mut found);

        let jvm = match JavaVM::from_raw(jvm_ptr as _) {
            Ok(jvm) => jvm,
            Err(_) => return Err(JVMError::JvmNotFound),
        };

        return Ok(jvm);
    }
}

fn get_minecraft(mut env: JNIEnv) -> jni::errors::Result<JObject> {
    let minecraft_class = match env.find_class("ave") {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let minecraft = match env.call_static_method(minecraft_class, "A", "()Lave;", &[]) {
        Ok(v) => v.l().unwrap(),
        Err(e) => return Err(e),
    };

    return Ok(minecraft);
}

fn attach() {
    unsafe {
        create_console();

        let jvm = get_jvm()
            .unwrap_or_else(|err| {
                message_box(err.to_string().as_str());
                std::process::exit(1);
            });

        let mut env = jvm.attach_current_thread()
            .unwrap_or_msgbox_and_exit(Some("Failed to get environment."));

        let minecraft = get_minecraft(env.unsafe_clone())
            .unwrap_or_msgbox_and_exit(None);
        
        let player = env.get_field(minecraft, "h", "Lbew;")
            .unwrap_or_msgbox_and_exit(Some("Failed to get player."))
            .l()
            .unwrap();

        loop {
            // Sprint forever.
            let _ = env.call_method(&player, "d", "(Z)V", &[JValue::from(true)]);
        
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[no_mangle]
extern "system" fn DllMain(
    _dll_module: HMODULE,
    call_reason: u32,
    _: *mut ()
) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => { std::thread::spawn(attach); },
        _ => ()
    };

    return true;
}
