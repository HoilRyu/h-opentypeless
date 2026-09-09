package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
public class ModelFailureTest {
    @Test public void temporaryFailuresDoNotDisableModels(){
        for(Throwable error:new Throwable[]{new OutOfMemoryError(),new java.io.IOException(),new IllegalStateException(),new java.util.concurrent.TimeoutException()})
            assertFalse(ModelFailure.blocks(ModelFailure.classify(error)));
        assertFalse(ModelFailure.blocks(null));
        assertFalse(ModelFailure.blocks(ModelFailure.QUALITY));
    }
    @Test public void knownBinaryIncompatibilityRequiresRecheck(){
        assertTrue(ModelFailure.blocks(ModelFailure.classify(new UnsatisfiedLinkError())));
        assertTrue(ModelFailure.blocks(ModelFailure.classify(new NoClassDefFoundError())));
    }
}
