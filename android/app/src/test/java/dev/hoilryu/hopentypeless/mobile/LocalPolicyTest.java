package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
public class LocalPolicyTest {
    @Test public void oldOrIncompatibleDevicesCannotRunBundledModels(){
        assertNotNull(Capability.hardware(28,true,12*Capability.GIB,6));
        assertNotNull(Capability.hardware(36,false,12*Capability.GIB,6));
        assertNotNull(Capability.hardware(36,true,4*Capability.GIB,8));
        assertNull(Capability.hardware(36,true,11*Capability.GIB,8));
    }
    @Test public void resumedDownloadMustMatchPinnedFileAndExactOffset(){
        assertTrue(ModelTransfer.validRange("bytes 40-99/100",40,100));
        assertFalse(ModelTransfer.validRange("bytes 0-99/100",40,100));
        assertFalse(ModelTransfer.validRange("bytes 40-199/200",40,100));
        assertFalse(ModelTransfer.validRange(null,40,100));
    }
}
