#include <SDL.h>
#include <accesskit.h>
#include <jni.h>
#include <stdbool.h>
#include <stdint.h>

extern accesskit_tree_update *build_initial_tree(void *userdata);
extern void do_action(accesskit_action_request *request, void *userdata);
extern void *get_window_state(void);
extern void *get_action_handler_state(void);
extern accesskit_tree_update *get_pending_update(void);

void android_request_accessibility_update(void) {
  JNIEnv *env = SDL_AndroidGetJNIEnv();
  if (env != NULL) {
    jclass cls = (*env)->FindClass(
        env, "dev/accesskit/sdl_example/AccessKitSDLActivity");
    if (cls != NULL) {
      jmethodID method = (*env)->GetStaticMethodID(
          env, cls, "requestAccessibilityUpdate", "()V");
      if (method != NULL) {
        (*env)->CallStaticVoidMethod(env, cls, method);
      }
      (*env)->DeleteLocalRef(env, cls);
    }
  }
}

static accesskit_android_adapter *g_adapter = NULL;

static accesskit_tree_update *pending_update_factory(void *userdata) {
  (void)userdata;
  return get_pending_update();
}

static jobject g_host = NULL;

JNIEXPORT jlong JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeCreateAccessKitAdapter(
    JNIEnv *env, jclass cls) {
  (void)env;
  (void)cls;
  g_adapter = accesskit_android_adapter_new();
  return (jlong)(uintptr_t)g_adapter;
}

JNIEXPORT void JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeFreeAccessKitAdapter(
    JNIEnv *env, jclass cls, jlong adapter) {
  (void)cls;
  (void)adapter;
  if (g_host != NULL) {
    (*env)->DeleteGlobalRef(env, g_host);
    g_host = NULL;
  }
  if (g_adapter != NULL) {
    accesskit_android_adapter_free(g_adapter);
    g_adapter = NULL;
  }
}

JNIEXPORT jobject JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeCreateAccessibilityNodeInfo(
    JNIEnv *env, jclass cls, jlong adapter_ptr, jobject host,
    jint virtualViewId) {
  (void)cls;
  (void)adapter_ptr;

  if (g_host == NULL && host != NULL) {
    g_host = (*env)->NewGlobalRef(env, host);
  }

  void *window_state = get_window_state();
  if (window_state == NULL || g_adapter == NULL) {
    return NULL;
  }

  return accesskit_android_adapter_create_accessibility_node_info(
      g_adapter, build_initial_tree, window_state, env, host, virtualViewId);
}

JNIEXPORT jobject JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeFindFocus(
    JNIEnv *env, jclass cls, jlong adapter_ptr, jobject host, jint focusType) {
  (void)cls;
  (void)adapter_ptr;

  void *window_state = get_window_state();
  if (window_state == NULL || g_adapter == NULL) {
    return NULL;
  }

  return accesskit_android_adapter_find_focus(
      g_adapter, build_initial_tree, window_state, env, host, focusType);
}

JNIEXPORT jboolean JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativePerformAction(
    JNIEnv *env, jclass cls, jlong adapter_ptr, jobject host,
    jint virtualViewId, jint action, jobject arguments) {
  (void)cls;
  (void)adapter_ptr;

  void *action_handler_state = get_action_handler_state();
  if (action_handler_state == NULL || g_adapter == NULL) {
    return JNI_FALSE;
  }

  accesskit_android_platform_action *platform_action =
      accesskit_android_platform_action_from_java(env, action, arguments);
  if (platform_action == NULL) {
    return JNI_FALSE;
  }

  accesskit_android_queued_events *events =
      accesskit_android_adapter_perform_action(g_adapter, do_action,
                                               action_handler_state,
                                               virtualViewId, platform_action);

  accesskit_android_platform_action_free(platform_action);

  if (events != NULL) {
    accesskit_android_queued_events_raise(events, env, host);
  }

  return JNI_TRUE;
}

JNIEXPORT void JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeUpdateAccessibility(
    JNIEnv *env, jclass cls, jlong adapter_ptr, jobject host) {
  (void)cls;
  (void)adapter_ptr;

  if (g_adapter == NULL || host == NULL) {
    return;
  }

  accesskit_android_queued_events *events =
      accesskit_android_adapter_update_if_active(g_adapter,
                                                 pending_update_factory, NULL);

  if (events != NULL) {
    accesskit_android_queued_events_raise(events, env, host);
  }
}

JNIEXPORT jboolean JNICALL
Java_dev_accesskit_sdl_1example_AccessKitSDLActivity_nativeOnHoverEvent(
    JNIEnv *env, jclass cls, jlong adapter_ptr, jobject host, jint action,
    jfloat x, jfloat y) {
  (void)cls;
  (void)adapter_ptr;

  void *window_state = get_window_state();
  if (window_state == NULL || g_adapter == NULL) {
    return JNI_FALSE;
  }

  accesskit_android_queued_events *events =
      accesskit_android_adapter_on_hover_event(g_adapter, build_initial_tree,
                                               window_state, action, x, y);

  if (events != NULL) {
    accesskit_android_queued_events_raise(events, env, host);
    return JNI_TRUE;
  }

  return JNI_FALSE;
}
