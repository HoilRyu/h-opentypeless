package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import android.media.*;
import android.os.*;
import android.system.*;
import java.io.IOException;
import java.util.concurrent.atomic.AtomicInteger;

/** One microphone, bounded PCM buffers, no audio file. Worker owns native cleanup. */
final class SttAudioStream implements AutoCloseable {
    final ParcelFileDescriptor input;
    private final ParcelFileDescriptor output;
    private final AtomicInteger peak=new AtomicInteger();
    private volatile boolean running;
    private boolean workerOwns;
    private AudioRecord audio;
    private RecordingLease lease;
    SttAudioStream(RecordingLease lease) throws IOException {
        if(Build.VERSION.SDK_INT<30)throw new IOException("마이크 스트림은 Android 11 이상 필요");
        ParcelFileDescriptor[] pipe=ParcelFileDescriptor.createPipe();input=pipe[0];output=pipe[1];this.lease=lease;
        try {Os.fcntlInt(output.getFileDescriptor(),OsConstants.F_SETFL,OsConstants.O_NONBLOCK);}
        catch(ErrnoException e){close();throw new IOException(e);}
    }
    void start(Context context,java.util.function.Consumer<String> error){
        if(context.checkSelfPermission(android.Manifest.permission.RECORD_AUDIO)!=android.content.pm.PackageManager.PERMISSION_GRANTED)throw new SecurityException("마이크 권한이 필요합니다.");
        try {
            int size=AudioRecord.getMinBufferSize(16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT);
            if(size<=0)throw new IllegalStateException("16kHz 녹음을 지원하지 않습니다.");
            audio=new AudioRecord(MediaRecorder.AudioSource.MIC,16000,AudioFormat.CHANNEL_IN_MONO,AudioFormat.ENCODING_PCM_16BIT,Math.max(size,8192));
            if(audio.getState()!=AudioRecord.STATE_INITIALIZED)throw new IllegalStateException("마이크 준비 실패");
            audio.startRecording();if(audio.getRecordingState()!=AudioRecord.RECORDSTATE_RECORDING)throw new IllegalStateException("마이크 시작 실패");
            running=true;Thread thread=new Thread(()->pump(error),"h-system-stt-audio");workerOwns=true;
            try {thread.start();}catch(RuntimeException|Error e){workerOwns=false;throw e;}
        }catch(RuntimeException|Error e){close();throw e;}
    }
    private void pump(java.util.function.Consumer<String> error){
        String failure=null;
        try {
            short[] samples=new short[320];byte[] bytes=new byte[640];
            while(running){
                int n=audio.read(samples,0,samples.length,AudioRecord.READ_NON_BLOCKING);
                if(n<0)throw new IOException("마이크 읽기 실패");
                if(n==0){Thread.sleep(5);continue;}
                int max=0;for(int i=0;i<n;i++){int v=samples[i];max=Math.max(max,Math.abs(v));bytes[i*2]=(byte)v;bytes[i*2+1]=(byte)(v>>8);}
                peak.accumulateAndGet(max,Math::max);
                long deadline=SystemClock.elapsedRealtime()+2000;int offset=0;
                while(running&&offset<n*2){
                    try {offset+=Os.write(output.getFileDescriptor(),bytes,offset,n*2-offset);}
                    catch(ErrnoException e){if(e.errno!=OsConstants.EAGAIN)throw e;if(SystemClock.elapsedRealtime()>deadline)throw new IOException("음성 인식기가 오디오 스트림을 읽지 않습니다.");Thread.sleep(5);}
                }
            }
        }catch(Exception e){if(running)failure="기본 STT 오디오 스트림 오류 · "+e.getMessage();}
        finally {running=false;releaseAudio();closeFd(output);}
        if(failure!=null)error.accept(failure);
    }
    int amplitude(){return peak.getAndSet(0);}
    void stop(){running=false;}
    private void releaseAudio(){if(audio!=null){try{audio.stop();}catch(RuntimeException ignored){}try{audio.release();}finally{audio=null;if(lease!=null){lease.close();lease=null;}}}else if(lease!=null){lease.close();lease=null;}}
    private static void closeFd(ParcelFileDescriptor fd){try{fd.close();}catch(IOException ignored){}}
    @Override public void close(){running=false;closeFd(input);if(!workerOwns){releaseAudio();closeFd(output);}}
}
