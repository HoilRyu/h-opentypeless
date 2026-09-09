package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import java.util.*;
import static org.junit.Assert.*;
public class SpeechSessionTest {
    static class Harness implements SpeechSession.Driver,SpeechSession.Output {
        List<Runnable> tasks=new ArrayList<>();int token,starts,stops,results;String result,error;
        SpeechSession session=new SpeechSession(this,this);
        public void start(int value){token=value;starts++;}
        public void stop(){stops++;}
        public void dispose(){}
        public void later(Runnable task,long ms){tasks.add(task);}
        public void result(String value){result=value;results++;}
        public void error(String value,String reason){result=value;error=reason;}
        void tick(){List<Runnable> copy=new ArrayList<>(tasks);tasks.clear();for(Runnable task:copy)task.run();}
    }
    @Test public void silenceAndFinalResultsContinueUntilButtonStop(){
        Harness h=new Harness();h.session.start();
        h.session.partial(h.token,"첫 문");h.session.result(h.token,"첫 문장");assertEquals(0,h.results);
        h.tick();h.session.silence(h.token);h.tick();assertEquals(3,h.starts);assertEquals(0,h.results);
        h.session.partial(h.token,"둘째 문장");h.session.stop();assertEquals(1,h.stops);assertEquals(0,h.results);
        h.session.result(h.token,"둘째 문장입니다");
        assertEquals("첫 문장 둘째 문장입니다",h.result);assertEquals(1,h.results);h.tick();assertEquals(3,h.starts);
    }
    @Test public void stopBetweenSegmentsPreventsRestartAndLateCallbacks(){
        Harness h=new Harness();h.session.start();int old=h.token;h.session.result(old,"첫 문장");h.session.stop();
        h.session.result(old,"늦은 중복");h.tick();assertEquals(1,h.starts);assertEquals("첫 문장",h.result);assertEquals(1,h.results);
    }
    @Test public void stopTimeoutPreservesLatestPartialOnlyOnce(){
        Harness h=new Harness();h.session.start();h.session.partial(h.token,"미완성");h.session.partial(h.token,"미완성 문장");h.session.stop();h.tick();h.session.result(h.token,"늦은 결과");h.session.stop();
        assertEquals("미완성 문장",h.result);assertEquals(1,h.results);
    }
    @Test public void cancelAndFatalErrorsNeverGenerateResults(){
        Harness h=new Harness();h.session.start();h.session.result(h.token,"보관");h.session.cancel();h.tick();assertEquals(1,h.starts);assertEquals(0,h.results);
        h=new Harness();h.session.start();h.session.partial(h.token,"오류 전 원문");h.session.failure(h.token,"권한 오류");h.tick();assertEquals("오류 전 원문",h.result);assertEquals("권한 오류",h.error);assertEquals(0,h.results);
    }
    @Test public void repeatedWordsAreIntentionalAndOldTokensCannotAppend(){
        Harness h=new Harness();h.session.start();int first=h.token;h.session.result(first,"다시");h.tick();h.session.result(first,"오래된 결과");h.session.result(h.token,"다시");h.session.stop();assertEquals("다시 다시",h.result);
    }
    @Test public void nativeSegmentsDoNotRestartAndStopCollectsFinalSegment(){
        Harness h=new Harness();h.session.start();int token=h.token;
        h.session.segment(token,"첫 문장");h.session.segment(token,"둘째 문장");h.tick();
        assertEquals(1,h.starts);assertEquals(0,h.results);
        h.session.stop();h.session.segment(token,"마지막 문장");h.session.segmentedEnd(token);h.tick();
        assertEquals("첫 문장 둘째 문장 마지막 문장",h.result);assertEquals(1,h.results);assertEquals(1,h.starts);
    }
    @Test public void nativeSegmentsPreservePartialOnTimeoutAndIgnoreLateResults(){
        Harness h=new Harness();h.session.start();h.session.segment(h.token,"완성");h.session.partial(h.token,"미완성");h.session.stop();h.tick();
        h.session.segment(h.token,"늦은 결과");h.session.segmentedEnd(h.token);
        assertEquals("완성 미완성",h.result);assertEquals(1,h.results);
    }
    @Test public void streamingSessionNeverRestartsWhenServiceEndsEarly(){
        Harness h=new Harness();h.session=new SpeechSession(h,h,true);h.session.start();
        h.session.segment(h.token,"보존할 문장");h.session.segmentedEnd(h.token);h.tick();
        assertEquals(1,h.starts);assertEquals(0,h.results);assertNotNull(h.error);assertEquals("보존할 문장",h.result);
    }
    @Test public void streamingStopClosesOnlyOnceAndKeepsFinalWords(){
        Harness h=new Harness();h.session=new SpeechSession(h,h,true);h.session.start();
        h.session.segment(h.token,"첫 문장");h.session.stop();h.session.segment(h.token,"끝 문장");h.session.segmentedEnd(h.token);h.tick();
        assertEquals(1,h.starts);assertEquals(1,h.stops);assertEquals(1,h.results);assertEquals("첫 문장 끝 문장",h.result);
    }
    @Test public void prolongedSilenceDoesNotFinishSession(){
        Harness h=new Harness();h.session.start();for(int i=0;i<100;i++){h.session.silence(h.token);h.tick();}
        assertEquals(0,h.results);assertEquals(101,h.starts);h.session.stop();h.session.silence(h.token);assertEquals("",h.result);
    }
}
