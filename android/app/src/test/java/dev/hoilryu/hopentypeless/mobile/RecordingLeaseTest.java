package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
public class RecordingLeaseTest {
    @Test public void stuckWorkerPreventsMoreRecordersAndDoubleRelease() {
        RecordingLease first=RecordingLease.acquire();
        try { assertThrows(IllegalStateException.class,RecordingLease::acquire); }
        finally {first.close();first.close();}
        try(RecordingLease second=RecordingLease.acquire()){
            assertNotNull(second);
            assertThrows(IllegalStateException.class,RecordingLease::acquire);
        }
    }
    @Test public void repeatedSessionsReturnTheirSlot() {
        for(int i=0;i<1000;i++){try(RecordingLease lease=RecordingLease.acquire()){assertNotNull(lease);}}
    }
}
