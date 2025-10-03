package dev.accesskit.sdl_example;

import android.content.Context;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.View;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;

import org.libsdl.app.SDLActivity;
import org.libsdl.app.SDLSurface;

public class AccessKitSDLActivity extends SDLActivity {
    private static long mAccessKitAdapter = 0;

    private static native long nativeCreateAccessKitAdapter();

    private static native void nativeFreeAccessKitAdapter(long adapter);

    private static native AccessibilityNodeInfo nativeCreateAccessibilityNodeInfo(
            long adapter, View host, int virtualViewId);

    private static native AccessibilityNodeInfo nativeFindFocus(
            long adapter, View host, int focusType);

    private static native boolean nativePerformAction(
            long adapter, View host, int virtualViewId, int action, Bundle arguments);

    private static native void nativeUpdateAccessibility(long adapter, View host);

    private static native boolean nativeOnHoverEvent(
            long adapter, View host, int action, float x, float y);

    @Override
    protected String[] getLibraries() {
        return new String[] {"SDL2", "hello_world"};
    }

    @Override
    protected SDLSurface createSDLSurface(Context context) {
        return new AccessKitSDLSurface(context);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        mAccessKitAdapter = nativeCreateAccessKitAdapter();
    }

    @Override
    protected void onDestroy() {
        if (mAccessKitAdapter != 0) {
            nativeFreeAccessKitAdapter(mAccessKitAdapter);
            mAccessKitAdapter = 0;
        }
        super.onDestroy();
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_BACK) {
            if (event.getAction() == KeyEvent.ACTION_UP) {
                finish();
            }
            return true;
        }
        return super.dispatchKeyEvent(event);
    }

    @Override
    public void setOrientationBis(int w, int h, boolean resizable, String hint) {
        setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
    }

    static long getAccessKitAdapter() {
        return mAccessKitAdapter;
    }

    public static void requestAccessibilityUpdate() {
        if (mSingleton != null && mSurface != null && mAccessKitAdapter != 0) {
            mSingleton.runOnUiThread(() -> nativeUpdateAccessibility(mAccessKitAdapter, mSurface));
        }
    }

    private class AccessKitSDLSurface extends SDLSurface {
        public AccessKitSDLSurface(Context context) {
            super(context);
            setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
        }

        @Override
        public boolean onHoverEvent(MotionEvent event) {
            long adapter = getAccessKitAdapter();
            if (adapter != 0
                    && nativeOnHoverEvent(
                            adapter, this, event.getAction(), event.getX(), event.getY())) {
                return true;
            }
            return super.onHoverEvent(event);
        }

        @Override
        public AccessibilityNodeProvider getAccessibilityNodeProvider() {
            long adapter = getAccessKitAdapter();
            if (adapter == 0) {
                return super.getAccessibilityNodeProvider();
            }
            return new AccessibilityNodeProvider() {
                @Override
                public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                    return nativeCreateAccessibilityNodeInfo(
                            adapter, AccessKitSDLSurface.this, virtualViewId);
                }

                @Override
                public AccessibilityNodeInfo findFocus(int focusType) {
                    return nativeFindFocus(adapter, AccessKitSDLSurface.this, focusType);
                }

                @Override
                public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                    return nativePerformAction(
                            adapter, AccessKitSDLSurface.this, virtualViewId, action, arguments);
                }
            };
        }
    }
}
