package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;

public class PressRepeaterTest {
    private static class Clock implements PressRepeater.Scheduler {
        Runnable pending;long delay;int count;
        public void post(Runnable task,long ms){pending=task;delay=ms;}
        public void remove(Runnable task){if(pending==task)pending=null;}
        void tick(){Runnable task=pending;pending=null;task.run();}
    }
    @Test public void tapAndHoldTiming(){
        Clock clock=new Clock();PressRepeater repeat=new PressRepeater(clock,()->clock.count++);
        repeat.start();assertEquals(1,clock.count);assertEquals(400,clock.delay);
        clock.tick();assertEquals(2,clock.count);assertEquals(70,clock.delay);
        clock.tick();assertEquals(3,clock.count);repeat.stop();assertNull(clock.pending);
    }
    @Test public void releaseAndNewPressRejectOldTicks(){
        Clock clock=new Clock();PressRepeater repeat=new PressRepeater(clock,()->clock.count++);
        repeat.start();Runnable stale=clock.pending;repeat.stop();stale.run();assertEquals(1,clock.count);
        repeat.start();stale.run();assertEquals(2,clock.count);clock.tick();assertEquals(3,clock.count);
    }
    @Test public void canceledDuringActionNeverSchedulesAgain(){
        Clock clock=new Clock();PressRepeater[] repeat=new PressRepeater[1];repeat[0]=new PressRepeater(clock,()->repeat[0].stop());
        repeat[0].start();assertNull(clock.pending);
    }
}
