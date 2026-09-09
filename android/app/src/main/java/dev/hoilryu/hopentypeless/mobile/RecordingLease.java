package dev.hoilryu.hopentypeless.mobile;
import java.util.concurrent.Semaphore;
import java.util.concurrent.atomic.AtomicBoolean;
/** A stuck recorder cannot accumulate more native recorders or cleanup threads. */
final class RecordingLease implements AutoCloseable {
    private static final Semaphore slot=new Semaphore(1);
    private final AtomicBoolean closed=new AtomicBoolean();
    static RecordingLease acquire(){
        if(!slot.tryAcquire())throw new IllegalStateException("이전 녹음 종료를 기다리는 중입니다.");
        return new RecordingLease();
    }
    @Override public void close(){if(closed.compareAndSet(false,true))slot.release();}
}
