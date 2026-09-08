package dev.hoilryu.hopentypeless.mobile;

import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.MediaRecorder;
import java.io.File;
import java.io.RandomAccessFile;

/** PCM WAV matches the desktop provider input; no ffmpeg dependency on either side. */
final class PcmRecorder {
    private final File file;
    private final android.content.Context context;
    private AudioRecord audio;
    private Thread thread;
    private volatile boolean running;
    private volatile int peak;
    private volatile RuntimeException failure;
    private volatile int bytesWritten;
    PcmRecorder(android.content.Context context, File file) { this.context=context; this.file=file; }
    void start() {
        if(context.checkSelfPermission(android.Manifest.permission.RECORD_AUDIO)!=android.content.pm.PackageManager.PERMISSION_GRANTED)throw new SecurityException("마이크 권한이 필요합니다.");
        int size=AudioRecord.getMinBufferSize(16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT);
        if(size<=0)throw new IllegalStateException("16kHz 녹음을 지원하지 않습니다.");
        audio=new AudioRecord(MediaRecorder.AudioSource.MIC,16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT,Math.max(size,8192));
        if(audio.getState()!=AudioRecord.STATE_INITIALIZED){release();throw new IllegalStateException("마이크를 시작하지 못했습니다.");}
        audio.startRecording();
        if(audio.getRecordingState()!=AudioRecord.RECORDSTATE_RECORDING){release();throw new IllegalStateException("마이크를 시작하지 못했습니다.");}
        running=true;
        final AudioRecord source=audio;
        thread=new Thread(()-> {
            try(RandomAccessFile out=new RandomAccessFile(file,"rw")) {
                out.setLength(0);out.write(Wav.header(0));
                short[] samples=new short[2048];byte[] data=new byte[4096];
                while(running && bytesWritten<Wav.MAX_PCM) {
                    int n=source.read(samples,0,samples.length,AudioRecord.READ_NON_BLOCKING);
                    if(n<0){if(running)throw new IllegalStateException("녹음 중 마이크 오류가 발생했습니다.");break;}
                    if(n==0){Thread.sleep(10);continue;}
                    n=Math.min(n,(Wav.MAX_PCM-bytesWritten)/2);
                    int level=0;
                    for(int i=0;i<n;i++){short v=samples[i];data[i*2]=(byte)v;data[i*2+1]=(byte)(v>>8);level=Math.max(level,Math.abs((int)v));}
                    peak=level;out.write(data,0,n*2);bytesWritten+=n*2;
                }
                out.seek(0);out.write(Wav.header(bytesWritten));
            }catch(Exception error){failure=new IllegalStateException("녹음 파일을 만들지 못했습니다.",error);}
        },"h-pcm-recording");
        thread.start();
    }
    int getMaxAmplitude(){int value=peak;peak=0;return value;}
    void stop() {
        finish();
        if(failure!=null)throw failure;
        if(bytesWritten<3200)throw new IllegalStateException("녹음이 너무 짧습니다.");
    }
    private void finish() {
        running=false;
        if(audio!=null){try{audio.stop();}catch(IllegalStateException ignored){}}
        if(thread!=null){
            try{thread.join(2000);}catch(InterruptedException e){Thread.currentThread().interrupt();throw new IllegalStateException(e);}
            if(thread.isAlive())throw new IllegalStateException("녹음 종료를 기다리는 중입니다.");
            thread=null;
        }
    }
    void reset(){release();}
    void release(){
        try{finish();}catch(RuntimeException error){failure=error;}
        if(thread==null && audio!=null){audio.release();audio=null;}
        else if(thread!=null && audio!=null){
            final Thread pending=thread;final AudioRecord pendingAudio=audio;
            Thread cleanup=new Thread(()->{try{pending.join();}catch(InterruptedException e){Thread.currentThread().interrupt();return;}pendingAudio.release();},"h-pcm-cleanup");
            cleanup.setDaemon(true);cleanup.start();audio=null;thread=null;
        }
    }
}
