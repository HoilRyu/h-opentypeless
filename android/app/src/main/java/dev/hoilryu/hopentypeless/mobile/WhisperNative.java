package dev.hoilryu.hopentypeless.mobile;
final class WhisperNative {
    static {System.loadLibrary("mobile_whisper");}
    static native byte[] transcribe(String model,String pcm,int threads,long[] timings) throws java.io.IOException;
}
