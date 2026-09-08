package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import java.util.*;
import static org.junit.Assert.*;
import android.view.KeyEvent;

public class EditingKeysTest {
    @Test public void editKeysSendBalancedPressAndRelease(){
        for(int key:new int[]{KeyEvent.KEYCODE_DPAD_LEFT,KeyEvent.KEYCODE_DPAD_RIGHT,KeyEvent.KEYCODE_DEL}){
            List<Integer> events=new ArrayList<>();
            assertTrue(EditingKeys.dispatch(key,(action,code)->{events.add(action);assertEquals(key,code);return true;}));
            assertEquals(Arrays.asList(KeyEvent.ACTION_DOWN,KeyEvent.ACTION_UP),events);
        }
    }
    @Test public void neverDispatchEnterVerticalOrOtherKeys(){
        for(int key:new int[]{KeyEvent.KEYCODE_ENTER,KeyEvent.KEYCODE_DPAD_UP,KeyEvent.KEYCODE_DPAD_DOWN,KeyEvent.KEYCODE_FORWARD_DEL,KeyEvent.KEYCODE_A})
            assertFalse(EditingKeys.dispatch(key,(action,code)->{fail("Unexpected key event");return true;}));
    }
    @Test public void releaseStillSentIfEditorRejectsDown(){
        List<Integer> events=new ArrayList<>();
        assertFalse(EditingKeys.dispatch(KeyEvent.KEYCODE_DEL,(action,code)->{events.add(action);return action==KeyEvent.ACTION_UP;}));
        assertEquals(Arrays.asList(KeyEvent.ACTION_DOWN,KeyEvent.ACTION_UP),events);
    }
}
