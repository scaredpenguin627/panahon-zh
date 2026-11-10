#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PANaHON 中文天气：egui 界面、Android 入口与本地通知桥接。
//!
//! 界面代码在 [`app`]，视觉风格在 [`theme`]，矢量天气图标在 [`icons`]，
//! 可复用的区块在 [`widgets`]。

mod app;
mod icons;
mod theme;
mod widgets;

pub use app::PanahonApp;

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "android")]
use std::thread;
#[cfg(target_os = "android")]
use std::time::Duration;

#[cfg(target_os = "android")]
static ANDROID_APP: OnceLock<winit::platform::android::activity::AndroidApp> = OnceLock::new();

#[cfg(target_os = "android")]
static NOTIFICATION_PERIODIC_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

#[cfg(target_os = "android")]
static NOTIFICATION_ONESHOT_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let _ = ANDROID_APP.set(app.clone());
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let result = eframe::run_native(
        "菲律宾天气",
        options,
        Box::new(|cc| Ok(Box::new(PanahonApp::new(cc)))),
    );
    if let Err(error) = result {
        eprintln!("Android UI failed to start: {error}");
    }
}

#[cfg(target_os = "android")]
pub(crate) fn window_insets_px() -> Option<(f32, f32)> {
    let app = ANDROID_APP.get()?;
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
    let raw_activity = app.activity_as_ptr() as jni::sys::jobject;

    vm.attach_current_thread(|env| -> jni::errors::Result<(f32, f32)> {
        let activity =
            unsafe { env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)? };
        let window = env
            .call_method(
                activity.as_ref(),
                jni::jni_str!("getWindow"),
                jni::jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let decor = env
            .call_method(
                &window,
                jni::jni_str!("getDecorView"),
                jni::jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let insets = env
            .call_method(
                &decor,
                jni::jni_str!("getRootWindowInsets"),
                jni::jni_sig!("()Landroid/view/WindowInsets;"),
                &[],
            )?
            .l()?;
        if insets.is_null() {
            return Ok((0.0, 0.0));
        }
        let top = env
            .call_method(
                &insets,
                jni::jni_str!("getSystemWindowInsetTop"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()?;
        let bottom = env
            .call_method(
                &insets,
                jni::jni_str!("getSystemWindowInsetBottom"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()?;
        Ok((top as f32, bottom as f32))
    })
    .ok()
}

#[cfg(target_os = "android")]
pub(crate) fn read_preference(key: &str) -> Option<String> {
    let app = ANDROID_APP.get()?.clone();
    let key = key.to_owned();
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
    vm.attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
        let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
        let activity =
            unsafe { env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)? };
        let name = env.new_string("panahon_settings")?;
        let prefs = env
            .call_method(
                activity.as_ref(),
                jni::jni_str!("getSharedPreferences"),
                jni::jni_sig!("(Ljava/lang/String;I)Landroid/content/SharedPreferences;"),
                &[
                    jni::objects::JValue::Object(&name),
                    jni::objects::JValue::Int(0),
                ],
            )?
            .l()?;
        let key = env.new_string(key)?;
        let default = jni::objects::JObject::null();
        let value = env
            .call_method(
                &prefs,
                jni::jni_str!("getString"),
                jni::jni_sig!("(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"),
                &[
                    jni::objects::JValue::Object(&key),
                    jni::objects::JValue::Object(&default),
                ],
            )?
            .l()?;
        if value.is_null() {
            return Ok(None);
        }
        let value = unsafe { jni::objects::JString::from_raw(&env, value.into_raw()) };
        Ok(Some(value.try_to_string(&env)?))
    })
    .ok()
    .flatten()
}

#[cfg(not(target_os = "android"))]
pub(crate) fn read_preference(_key: &str) -> Option<String> {
    None
}

#[cfg(target_os = "android")]
pub(crate) fn write_preference(key: &str, value: &str) {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    let key = key.to_owned();
    let value = value.to_owned();
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let name = env.new_string("panahon_settings")?;
            let prefs = env
                .call_method(
                    activity.as_ref(),
                    jni::jni_str!("getSharedPreferences"),
                    jni::jni_sig!("(Ljava/lang/String;I)Landroid/content/SharedPreferences;"),
                    &[
                        jni::objects::JValue::Object(&name),
                        jni::objects::JValue::Int(0),
                    ],
                )?
                .l()?;
            let editor = env
                .call_method(
                    &prefs,
                    jni::jni_str!("edit"),
                    jni::jni_sig!("()Landroid/content/SharedPreferences$Editor;"),
                    &[],
                )?
                .l()?;
            let key = env.new_string(key)?;
            let value = env.new_string(value)?;
            env.call_method(
                &editor,
                jni::jni_str!("putString"),
                jni::jni_sig!(
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;"
                ),
                &[
                    jni::objects::JValue::Object(&key),
                    jni::objects::JValue::Object(&value),
                ],
            )?;
            env.call_method(&editor, jni::jni_str!("apply"), jni::jni_sig!("()V"), &[])?;
            Ok(())
        });
    }));
}

#[cfg(not(target_os = "android"))]
pub(crate) fn write_preference(_key: &str, _value: &str) {}

#[cfg(target_os = "android")]
pub(crate) fn request_notification_permission() {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let permission = env.new_string("android.permission.POST_NOTIFICATIONS")?;
            let string_class = env.find_class(jni::jni_str!("java/lang/String"))?;
            let permissions =
                env.new_object_array(1, string_class, jni::objects::JObject::null())?;
            permissions.set_element(&env, 0, &permission)?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("requestPermissions"),
                jni::jni_sig!("([Ljava/lang/String;I)V"),
                &[
                    jni::objects::JValue::Object(&permissions),
                    jni::objects::JValue::Int(1001),
                ],
            )?;
            Ok(())
        });
    }));
}

#[cfg(target_os = "android")]
pub(crate) fn start_local_notification_scheduler() -> bool {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return false;
    };
    let slot = NOTIFICATION_PERIODIC_STOP.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().expect("notification scheduler mutex poisoned");
    if guard.is_some() {
        return false;
    }
    let stop = Arc::new(AtomicBool::new(false));
    *guard = Some(stop.clone());
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            if wait_for_notification(&stop, Duration::from_secs(30 * 60)) {
                break;
            }
            post_notification_with_app(
                app.clone(),
                "菲律宾天气".into(),
                "打开应用查看 PAGASA 最新预警。".into(),
                2001,
            );
        }
    });
    true
}

#[cfg(target_os = "android")]
pub(crate) fn stop_local_notification_scheduler() {
    if let Some(slot) = NOTIFICATION_PERIODIC_STOP.get() {
        if let Some(stop) = slot
            .lock()
            .expect("notification scheduler mutex poisoned")
            .take()
        {
            stop.store(true, Ordering::Relaxed);
        }
    }
    if let Some(slot) = NOTIFICATION_ONESHOT_STOP.get() {
        if let Some(stop) = slot
            .lock()
            .expect("notification timer mutex poisoned")
            .take()
        {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "android")]
fn wait_for_notification(stop: &AtomicBool, duration: Duration) -> bool {
    let mut elapsed = Duration::ZERO;
    while elapsed < duration {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        thread::sleep(Duration::from_secs(1));
        elapsed += Duration::from_secs(1);
    }
    stop.load(Ordering::Relaxed)
}

#[cfg(target_os = "android")]
pub(crate) fn post_test_notification() {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    post_notification_with_app(
        app,
        "菲律宾天气".into(),
        "这是一条本地通知测试。".into(),
        1001,
    );
}

#[cfg(target_os = "android")]
fn post_notification_with_app(
    app: winit::platform::android::activity::AndroidApp,
    title: String,
    body: String,
    notification_id: i32,
) {
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let service_name = env.new_string("notification")?;
            let manager = env
                .call_method(
                    activity.as_ref(),
                    jni::jni_str!("getSystemService"),
                    jni::jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[jni::objects::JValue::Object(&service_name)],
                )?
                .l()?;
            let channel_id = env.new_string("panahon_alerts")?;
            let channel_name = env.new_string("天气提醒")?;
            let channel = env.new_object(
                jni::jni_str!("android/app/NotificationChannel"),
                jni::jni_sig!("(Ljava/lang/String;Ljava/lang/CharSequence;I)V"),
                &[
                    jni::objects::JValue::Object(&channel_id),
                    jni::objects::JValue::Object(&channel_name),
                    jni::objects::JValue::Int(3),
                ],
            )?;
            env.call_method(
                &manager,
                jni::jni_str!("createNotificationChannel"),
                jni::jni_sig!("(Landroid/app/NotificationChannel;)V"),
                &[jni::objects::JValue::Object(&channel)],
            )?;
            let builder = env.new_object(
                jni::jni_str!("android/app/Notification$Builder"),
                jni::jni_sig!("(Landroid/content/Context;Ljava/lang/String;)V"),
                &[
                    jni::objects::JValue::Object(activity.as_ref()),
                    jni::objects::JValue::Object(&channel_id),
                ],
            )?;
            let title = env.new_string(title)?;
            let body = env.new_string(body)?;
            env.call_method(
                &builder,
                jni::jni_str!("setContentTitle"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Object(&title)],
            )?;
            env.call_method(
                &builder,
                jni::jni_str!("setContentText"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Object(&body)],
            )?;
            let icon = env
                .get_static_field(
                    jni::jni_str!("android/R$drawable"),
                    jni::jni_str!("sym_def_app_icon"),
                    jni::jni_sig!("I"),
                )?
                .i()?;
            env.call_method(
                &builder,
                jni::jni_str!("setSmallIcon"),
                jni::jni_sig!("(I)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Int(icon)],
            )?;
            let notification = env
                .call_method(
                    &builder,
                    jni::jni_str!("build"),
                    jni::jni_sig!("()Landroid/app/Notification;"),
                    &[],
                )?
                .l()?;
            env.call_method(
                &manager,
                jni::jni_str!("notify"),
                jni::jni_sig!("(ILandroid/app/Notification;)V"),
                &[
                    jni::objects::JValue::Int(notification_id),
                    jni::objects::JValue::Object(&notification),
                ],
            )?;
            Ok(())
        });
    }));
}

#[cfg(target_os = "android")]
pub(crate) fn open_official_map(url: &str) {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    let url = url.to_owned();
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let action = env.new_string("android.intent.action.VIEW")?;
            let intent = env.new_object(
                jni::jni_str!("android/content/Intent"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[jni::objects::JValue::Object(&action)],
            )?;
            let url = env.new_string(url)?;
            let uri = env
                .call_static_method(
                    jni::jni_str!("android/net/Uri"),
                    jni::jni_str!("parse"),
                    jni::jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
                    &[jni::objects::JValue::Object(&url)],
                )?
                .l()?;
            env.call_method(
                &intent,
                jni::jni_str!("setData"),
                jni::jni_sig!("(Landroid/net/Uri;)Landroid/content/Intent;"),
                &[jni::objects::JValue::Object(&uri)],
            )?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("startActivity"),
                jni::jni_sig!("(Landroid/content/Intent;)V"),
                &[jni::objects::JValue::Object(&intent)],
            )?;
            Ok(())
        });
    }));
}
