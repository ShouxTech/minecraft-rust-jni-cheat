use jni::JavaVM;
use jni::objects::JValue;
use jni::sys::{jsize, jint};
use windows::Win32::System::Console::AllocConsole;
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, s};
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, core::*, Win32::System::LibraryLoader::* };

type JNIGetCreatedJavaVMs = fn (vmBuf: *mut *mut JavaVM, bufLen: jsize, nVMs: *mut jsize) -> jint;

fn message_box(text: &str) {
    unsafe {
        let null_terminated_text = text.to_owned() + "\0";

        MessageBoxA(HWND(0), PCSTR(null_terminated_text.as_ptr()), s!("dll"), Default::default());
    }
}

fn attach() {
    unsafe {
        AllocConsole(); // Doesn't actaully redirect output (println!) to console because I couldn't get freopen to work.

        let jvm_dll = GetModuleHandleA(s!("jvm.dll")).unwrap_or(HMODULE(0));
        if jvm_dll == HMODULE(0) {
            message_box("Failed to get JVM DLL.");
            return;
        }

        let jni_get_created_java_vms_ptr = GetProcAddress(jvm_dll, s!("JNI_GetCreatedJavaVMs")).unwrap();

        let jni_get_created_java_vms: JNIGetCreatedJavaVMs = std::mem::transmute(jni_get_created_java_vms_ptr);

        let mut jvm_ptr: *mut JavaVM = std::ptr::null_mut();
        let mut found: jsize = 0;

        jni_get_created_java_vms(&mut jvm_ptr, 1, &mut found);

        let jvm = JavaVM::from_raw(jvm_ptr as _)
            .unwrap_or_else(|_| {
                message_box("Failed to get JVM.");
                std::process::exit(1);
            });

        let mut env = jvm.attach_current_thread()
            .unwrap_or_else(|_| {
                message_box("Failed to get environment.");
                std::process::exit(1);
            });

        let minecraft_class = env.find_class("ave")
            .unwrap_or_else(|_| {
                message_box("Failed to get Minecraft class.");
                std::process::exit(1);
            });

        let minecraft = env.call_static_method(minecraft_class, "A", "()Lave;", &[])
            .unwrap()
            .l()
            .unwrap();

        let player = env.get_field(minecraft, "h", "Lbew;")
            .unwrap_or_else(|_| {
                message_box("Failed to get player.");
                std::process::exit(1);
            })
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
#[allow(unused_variables)]
extern "system" fn DllMain(
    dll_module: HMODULE,
    call_reason: u32,
    _: *mut ()
) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        _ => ()
    }

    return true;
}