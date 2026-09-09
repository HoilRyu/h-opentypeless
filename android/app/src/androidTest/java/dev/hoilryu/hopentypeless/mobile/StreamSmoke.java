package dev.hoilryu.hopentypeless.mobile;

import android.app.Instrumentation;
import android.os.*;
import android.speech.*;
import android.system.*;
import java.util.concurrent.*;

/** Sends synthetic silence via the same streaming Intent; no app microphone capture. */
final class StreamSmoke {
    static String run(Instrumentation test,boolean speech) throws Exception {
        ParcelFileDescriptor[] pipe=ParcelFileDescriptor.createPipe();
        Os.fcntlInt(pipe[1].getFileDescriptor(),OsConstants.F_SETFL,OsConstants.O_NONBLOCK);
        CountDownLatch done=new CountDownLatch(1);SpeechRecognizer[] recognizer={null};
        StringBuilder transcript=new StringBuilder();
        long start=SystemClock.elapsedRealtime();long[] ended={0};int[] error={0},segments={0};
        test.runOnMainSync(()->{
            SpeechRecognizer r=SpeechRecognizer.createOnDeviceSpeechRecognizer(test.getTargetContext());recognizer[0]=r;
            r.setRecognitionListener(new RecognitionListener(){
                public void onReadyForSpeech(Bundle b){}public void onBeginningOfSpeech(){}public void onRmsChanged(float v){}public void onBufferReceived(byte[] b){}public void onEndOfSpeech(){}public void onPartialResults(Bundle b){}public void onEvent(int t,Bundle b){}
                private void collect(Bundle b){java.util.ArrayList<String> values=b.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION);if(values!=null&&!values.isEmpty())transcript.append(values.get(0)).append(" ");}
                public void onResults(Bundle b){collect(b);ended[0]=SystemClock.elapsedRealtime();done.countDown();}
                public void onSegmentResults(Bundle b){collect(b);segments[0]++;}
                public void onEndOfSegmentedSession(){ended[0]=SystemClock.elapsedRealtime();done.countDown();}
                public void onError(int code){error[0]=code;ended[0]=SystemClock.elapsedRealtime();done.countDown();}
            });r.startListening(SystemStt.streamIntent(pipe[0]));
        });
        try {
            byte[] recording=speech?java.nio.file.Files.readAllBytes(new java.io.File(test.getTargetContext().getCacheDir(),"stream-test.pcm").toPath()):new byte[384000];
            byte[] silence=new byte[640];int frames=(recording.length+639)/640;
            for(int frame=0;frame<frames&&done.getCount()>0;frame++){
                java.util.Arrays.fill(silence,(byte)0);System.arraycopy(recording,frame*640,silence,0,Math.min(640,recording.length-frame*640));
                int offset=0;long deadline=SystemClock.elapsedRealtime()+2000;
                while(offset<silence.length){try{offset+=Os.write(pipe[1].getFileDescriptor(),silence,offset,silence.length-offset);}catch(ErrnoException e){if(e.errno!=OsConstants.EAGAIN)throw e;if(SystemClock.elapsedRealtime()>deadline)throw new AssertionError("stream not consumed");Thread.sleep(5);}}
                Thread.sleep(20);
            }
            long closed=SystemClock.elapsedRealtime();pipe[1].close();
            if(!done.await(10,TimeUnit.SECONDS))throw new AssertionError("no EOF callback");
            if(closed-start<frames*20L)throw new AssertionError("early end: error="+error[0]+" elapsedMs="+(ended[0]-start));
            if(error[0]!=0&&error[0]!=SpeechRecognizer.ERROR_NO_MATCH)throw new AssertionError("stream recognition error="+error[0]);
            if(ended[0]<closed-100)throw new AssertionError("recognizer ended before stream close at "+(ended[0]-start)+"ms");
            int mentions=transcript.toString().split("문장",-1).length-1;
            if(speech&&mentions<2)throw new AssertionError("speech content missing across pause; fixture word mentions="+mentions+" segments="+segments[0]);
            return "stream stayed open through "+(frames*20L)+"ms input; EOF ended session; terminalError="+error[0]+" segments="+segments[0]+" elapsedMs="+(ended[0]-start);
        }finally{
            test.runOnMainSync(()->{recognizer[0].cancel();recognizer[0].destroy();});pipe[0].close();pipe[1].close();
        }
    }
}
