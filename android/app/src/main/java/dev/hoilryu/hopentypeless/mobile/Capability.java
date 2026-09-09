package dev.hoilryu.hopentypeless.mobile;

import android.app.ActivityManager;
import android.content.Context;
import android.os.Build;
import android.os.PowerManager;
import android.speech.SpeechRecognizer;
import java.util.Arrays;

final class Capability {
    static final long GIB=1024L*1024*1024;
    static ActivityManager.MemoryInfo memory(Context c){ActivityManager.MemoryInfo m=new ActivityManager.MemoryInfo();((ActivityManager)c.getSystemService(Context.ACTIVITY_SERVICE)).getMemoryInfo(m);return m;}
    static String systemService(Context c){return Build.VERSION.SDK_INT<31?"Android 12 이상 필요":!SpeechRecognizer.isOnDeviceRecognitionAvailable(c)?"기본 온디바이스 STT 서비스 없음":null;}
    static String system(Context c){
        String reason=systemService(c);if(reason!=null)return reason;
        String state=LocalConfig.prefs(c).getString("system-ko-state","unknown");
        if(state.equals("missing"))return "한국어 모델 준비 필요 · 로컬 설정에서 확인";
        if(state.equals("unsupported"))return "기본 STT 한국어 미지원 · 로컬 설정에서 재확인";
        return null;
    }
    // Conservative first-release policy, not a hardware benchmark or manufacturer guarantee.
    static String hardware(int sdk,boolean arm64,long total, double minimum){
        if(sdk<31)return "로컬 모델은 Android 12 이상 필요";
        if(!arm64)return "64비트 ARM 기기 필요";
        if(total<minimum*GIB*.9)return "메모리 부족 · 이 모델은 RAM "+(int)minimum+"GB 이상 대상";
        return null;
    }
    static String hardware(Context c,ModelCatalog.Model m){return hardware(Build.VERSION.SDK_INT,Arrays.asList(Build.SUPPORTED_ABIS).contains("arm64-v8a"),memory(c).totalMem,m.ramGb);}
    static String ready(Context c,ModelCatalog.Model m){
        String reason=hardware(c,m);if(reason!=null)return reason;
        if(!m.installed(c))return "모델 설치 필요";
        reason=LocalConfig.prefs(c).getString("failure-"+m.id,null);if(reason!=null&&ModelFailure.blocks(LocalConfig.prefs(c).getString("failure-kind-"+m.id,null)))return "실행 확인 실패 · 설정에서 재검사: "+reason;
        ActivityManager.MemoryInfo info=memory(c);
        if(info.lowMemory||info.availMem<m.availableGb*GIB)return "현재 여유 메모리 부족 · 다른 앱 종료 후 다시 시도";
        if(Build.VERSION.SDK_INT>=29&&((PowerManager)c.getSystemService(Context.POWER_SERVICE)).getCurrentThermalStatus()>=PowerManager.THERMAL_STATUS_SEVERE)return "기기가 뜨겁습니다 · 식힌 뒤 다시 시도";
        return null;
    }
    static String stt(Context c){return LocalConfig.stt(c).equals("system")?system(c):ready(c,ModelCatalog.get(c,LocalConfig.stt(c)));}
    static String selected(Context c){String reason=stt(c);if(reason!=null)return reason;return LocalConfig.llm(c).equals("none")?null:ready(c,ModelCatalog.get(c,LocalConfig.llm(c)));}
}
