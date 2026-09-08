package dev.hoilryu.hopentypeless.mobile;

/** One immediate key, then repeat while held. Stale ticks cannot cross input sessions. */
final class PressRepeater {
    interface Scheduler { void post(Runnable task,long delay); void remove(Runnable task); }
    private final Scheduler scheduler;
    private final Runnable action;
    private Runnable tick;
    private long generation;
    private boolean pressed;
    PressRepeater(Scheduler scheduler,Runnable action){this.scheduler=scheduler;this.action=action;}
    void start(){
        stop();pressed=true;final long current=generation;
        tick=()->{if(!pressed||current!=generation)return;action.run();if(pressed&&current==generation)scheduler.post(tick,70);};
        action.run();if(pressed&&current==generation)scheduler.post(tick,400);
    }
    void stop(){pressed=false;generation++;if(tick!=null)scheduler.remove(tick);tick=null;}
}
