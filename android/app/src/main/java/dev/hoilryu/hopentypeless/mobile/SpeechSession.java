package dev.hoilryu.hopentypeless.mobile;

/** One button-controlled dictation composed of recognizer-owned utterances. Main thread only. */
final class SpeechSession {
    interface Driver { void start(int token); void stop(); void dispose(); void later(Runnable task,long ms); }
    interface Output { void result(String text); void error(String text,String reason); }
    private final Driver driver;
    private final Output output;
    private final boolean singleSession;
    private final StringBuilder committed=new StringBuilder();
    private String partial="";
    private boolean listening,stopping,done;
    private int token;
    SpeechSession(Driver driver,Output output){this(driver,output,false);}
    SpeechSession(Driver driver,Output output,boolean singleSession){this.driver=driver;this.output=output;this.singleSession=singleSession;}
    void start(){if(!done&&!stopping&&!listening){listening=true;partial="";driver.start(++token);}}
    boolean accepts(int value){return !done&&listening&&token==value;}
    void partial(int value,String text){if(accepts(value)&&!text.trim().isEmpty())partial=text.trim();}
    // A segment is final text, not the end of the recognizer session.
    void segment(int value,String text){if(!accepts(value))return;append(text.trim().isEmpty()?partial:text.trim());partial="";}
    void segmentedEnd(int value){if(!accepts(value))return;append(partial);partial="";next();}
    void result(int value,String text){
        if(!accepts(value))return;
        append(text.trim().isEmpty()?partial:text.trim());partial="";next();
    }
    void silence(int value){if(!accepts(value))return;append(partial);partial="";next();}
    private void next(){
        listening=false;driver.dispose();
        if(stopping)finish();else if(singleSession){done=true;output.error(text(),"기본 STT가 연속 입력을 종료했습니다. 원문을 유지합니다. 계속 발생하면 Whisper를 선택해 주세요.");}else driver.later(this::start,250);
    }
    void failure(int value,String reason){
        if(!accepts(value))return;
        done=true;listening=false;driver.dispose();output.error(text(),reason);
    }
    void stop(){
        if(done||stopping)return;stopping=true;
        if(!listening){finish();return;}
        driver.stop();
        // Some services never deliver a final result after stop. Preserve the latest partial.
        driver.later(()->{if(!done&&stopping)finish();},5000);
    }
    private void finish(){if(done)return;done=true;listening=false;driver.dispose();output.result(text());}
    void cancel(){if(done)return;done=true;listening=false;driver.dispose();}
    String text(){return committed+(partial.isEmpty()?"":(committed.length()==0?"":" ")+partial);}
    private void append(String text){if(text.isEmpty())return;if(committed.length()>0)committed.append(' ');committed.append(text);}
}
