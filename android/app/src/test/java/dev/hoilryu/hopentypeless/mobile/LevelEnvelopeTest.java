package dev.hoilryu.hopentypeless.mobile;

import org.junit.Test;
import static org.junit.Assert.*;

public class LevelEnvelopeTest {
    @Test public void sparseSamplesAnimateBetweenCallbacksAndThenDecay(){
        LevelEnvelope e=new LevelEnvelope();e.sample(1,1000);
        float first=e.frame(1000),second=e.frame(1016);
        assertTrue(first>0&&second>first&&second<1);
        float peak=e.frame(1200);
        for(long t=1320;t<=2400;t+=16)e.frame(t);
        assertTrue(e.frame(2416)<peak*.01f);
    }
    @Test public void refreshRateDoesNotChangeResponse(){
        LevelEnvelope a=new LevelEnvelope(),b=new LevelEnvelope();
        a.frame(1000);b.frame(1000);a.sample(1,1000);b.sample(1,1000);
        for(long t=1016;t<=1256;t+=16)a.frame(t);
        for(long t=1008;t<=1256;t+=8)b.frame(t);
        assertEquals(a.frame(1256),b.frame(1256),.0001f);
    }
    @Test public void resetAndInvalidLevelsCannotLeaveStuckBars(){
        LevelEnvelope e=new LevelEnvelope();e.sample(1,1000);assertTrue(e.frame(1000)>0);
        e.reset();assertEquals(0,e.frame(1016),0);
        e.sample(Float.NaN,1020);assertEquals(0,e.frame(1032),0);
        e.sample(-5,1040);assertEquals(0,e.frame(1048),0);
        e.sample(50,1050);assertTrue(e.frame(1064)<=1);
    }
}
