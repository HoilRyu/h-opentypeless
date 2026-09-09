package dev.hoilryu.hopentypeless.mobile;

/** Smooth measured levels at display cadence; missing samples decay rather than freeze. */
final class LevelEnvelope {
    private float target,value;
    private long sampled,frame=-1;
    void sample(float level,long now){target=Float.isFinite(level)?Math.max(0,Math.min(1,level)):0;sampled=now;}
    float frame(long now){
        long dt=frame<0?16:Math.max(0,Math.min(100,now-frame));frame=now;
        float goal=now-sampled>300?0:target;
        value+=(goal-value)*(float)(1-Math.exp(-dt/(goal>value?65.0:180.0)));
        return value;
    }
    void reset(){target=value=0;sampled=0;frame=-1;}
}
