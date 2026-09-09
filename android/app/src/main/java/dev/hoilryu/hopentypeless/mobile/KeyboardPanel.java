package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.*;
import android.graphics.drawable.RippleDrawable;
import android.view.*;
import android.widget.*;

/** A fixed action area below independently scrollable recognition and correction results. */
final class KeyboardPanel extends LinearLayout {
    interface Actions {void mode();void record();void insert(String value);void clear();void keyboard();void settings();void moveLeft();void moveRight();void backspace();}
    private static final int BG=AppUi.BG,INK=AppUi.INK,MUTED=AppUi.MUTED,BLUE=AppUi.ACCENT;
    private final Button mode,primary,insert,originalChoice,clear;
    private final TextView badge,status,rawView,correctedView,idleText;
    private final ImageButton left,right,backspace,settings;
    private final LinearLayout results,idle,rawCard,correctedCard;
    private final LevelView level;
    private String original="",corrected="";
    private boolean showOriginal,allowInsertion,expanded;
    private int bottomInset;
    private long displayedSeconds=-1;
    KeyboardPanel(Context c,Actions actions){
        super(c);setOrientation(VERTICAL);setBackground(AppUi.shape(c,BG,22));bottomInset=dp(56);padding();
        setOnApplyWindowInsetsListener((v,i)->{int n=android.os.Build.VERSION.SDK_INT>=30?i.getInsetsIgnoringVisibility(WindowInsets.Type.navigationBars()).bottom:i.getStableInsetBottom();
            if(android.os.Build.VERSION.SDK_INT>=29)n=Math.max(n,i.getMandatorySystemGestureInsets().bottom);
            // Some OEMs report zero navigation inset while drawing IME switch/hide controls.
            // Keep a separate 48dp system-control lane and an 8dp gap below our toolbar.
            bottomInset=Math.max(dp(48),n)+dp(8);padding();requestLayout();return i;});
        LinearLayout header=new LinearLayout(c);header.setGravity(Gravity.CENTER_VERTICAL);
        mode=button("H-OpenTypeless",actions::mode);header.addView(mode,new LayoutParams(0,dp(48),1));
        badge=text("준비",12,BLUE);badge.setGravity(Gravity.CENTER);header.addView(badge,new LayoutParams(dp(64),dp(48)));
        settings=icon(KeyIcon.Kind.SETTINGS,"사용 설정",actions::settings);header.addView(settings,new LayoutParams(dp(48),dp(48)));addView(header);
        ScrollView scroll=new ScrollView(c);scroll.setFillViewport(true);LayoutParams contentSize=new LayoutParams(-1,0,1);contentSize.topMargin=dp(8);addView(scroll,contentSize);
        LinearLayout content=AppUi.column(c);scroll.addView(content);
        results=AppUi.column(c);content.addView(results);
        rawCard=resultCard("인식 원문");rawView=AppUi.text(rawCard,"",14,MUTED);rawView.setPadding(0,0,0,0);
        correctedCard=resultCard("다듬은 문장");correctedView=AppUi.text(correctedCard,"",16,INK);correctedView.setPadding(0,0,0,0);
        idle=AppUi.column(c);idle.setGravity(Gravity.CENTER);content.addView(idle,new LayoutParams(-1,-1));
        level=new LevelView(c);idle.addView(level,new LayoutParams(-1,dp(48)));
        idleText=text("목소리를 글로 바꿔보세요",16,INK);idleText.setGravity(Gravity.CENTER);idle.addView(idleText,new LayoutParams(-1,dp(48)));
        status=text("종료 후 내용을 확인하고 삽입하세요.",12,MUTED);status.setMaxLines(2);status.setAccessibilityLiveRegion(ACCESSIBILITY_LIVE_REGION_POLITE);addView(status,new LayoutParams(-1,dp(40)));
        LinearLayout actionRow=new LinearLayout(c);
        primary=button("말하기",actions::record);AppUi.style(primary,true);actionRow.addView(primary,new LayoutParams(0,dp(56),1));
        insert=button("문장 삽입",()->actions.insert(showOriginal?original:corrected));AppUi.style(insert,true);LayoutParams ip=new LayoutParams(0,dp(56),2);ip.leftMargin=dp(12);actionRow.addView(insert,ip);addView(actionRow);
        LinearLayout tools=new LinearLayout(c);tools.setGravity(Gravity.CENTER_VERTICAL);
        tools.addView(icon(KeyIcon.Kind.KEYBOARD,"일반 키보드 전환",actions::keyboard),new LayoutParams(dp(48),dp(48)));
        left=icon(KeyIcon.Kind.LEFT,"커서 왼쪽",actions::moveLeft);right=icon(KeyIcon.Kind.RIGHT,"커서 오른쪽",actions::moveRight);backspace=icon(KeyIcon.Kind.BACKSPACE,"한 글자 삭제",actions::backspace);
        tools.addView(left,toolSize());tools.addView(right,toolSize());tools.addView(backspace,toolSize());
        originalChoice=button("원문 사용",()->{showOriginal=!showOriginal;renderText();});LayoutParams choiceSize=new LayoutParams(-1,dp(48));choiceSize.topMargin=dp(8);results.addView(originalChoice,choiceSize);
        rawCard.setOnClickListener(v->{if(allowInsertion&&!original.isEmpty()){showOriginal=true;renderText();}});correctedCard.setOnClickListener(v->{if(allowInsertion&&!corrected.isEmpty()){showOriginal=false;renderText();}});
        View spacer=new View(c);tools.addView(spacer,new LayoutParams(0,1,1));
        clear=button("비우기",actions::clear);LayoutParams clearSize=new LayoutParams(dp(56),dp(48));clearSize.leftMargin=dp(8);tools.addView(clear,clearSize);LayoutParams toolRowSize=new LayoutParams(-1,dp(48));toolRowSize.topMargin=dp(12);addView(tools,toolRowSize);render("","",false,false,false);
    }
    private void padding(){setPadding(dp(16),dp(12),dp(16),bottomInset);}
    private LayoutParams toolSize(){LayoutParams lp=new LayoutParams(dp(48),dp(48));lp.leftMargin=dp(8);return lp;}
    private LinearLayout resultCard(String label){LinearLayout card=AppUi.column(getContext());card.setPadding(dp(10),dp(8),dp(10),dp(8));LayoutParams lp=new LayoutParams(-1,-2);lp.topMargin=dp(6);results.addView(card,lp);TextView heading=AppUi.text(card,label,12,MUTED);heading.setPadding(0,0,0,dp(3));return card;}
    private int dp(int n){return Ui.dp(getContext(),n);}
    private TextView text(String s,int size,int color){TextView t=new TextView(getContext());t.setText(s);t.setTextSize(size);t.setTextColor(color);t.setGravity(Gravity.CENTER_VERTICAL);return t;}
    private Button button(String label,Runnable action){Button b=new Button(getContext());b.setText(label);b.setAllCaps(false);b.setTextSize(13);b.setPadding(dp(6),0,dp(6),0);b.setMinWidth(0);b.setMinimumWidth(0);b.setMinHeight(0);b.setMinimumHeight(0);b.setTextColor(INK);b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x332abba7),AppUi.shape(getContext(),BG,12),null));b.setOnClickListener(v->action.run());return b;}
    private ImageButton icon(KeyIcon.Kind kind,String label,Runnable run){boolean repeat=kind==KeyIcon.Kind.LEFT||kind==KeyIcon.Kind.RIGHT||kind==KeyIcon.Kind.BACKSPACE;ImageButton b=repeat?new RepeatKeyButton(getContext()):new ImageButton(getContext());b.setImageDrawable(new KeyIcon(kind,MUTED));b.setPadding(dp(12),dp(12),dp(12),dp(12));b.setScaleType(ImageView.ScaleType.FIT_CENTER);b.setContentDescription(label+(repeat?" · 길게 누르면 반복":""));b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x332abba7),AppUi.shape(getContext(),BG,10),null));b.setOnClickListener(v->{v.performHapticFeedback(HapticFeedbackConstants.KEYBOARD_TAP);run.run();});return b;}
    @Override protected void onMeasure(int width,int height){float scale=Math.max(1,Math.min(1.6f,getResources().getConfiguration().fontScale));int desired=Math.round(dp(expanded?430:342)*scale)+bottomInset;boolean landscape=getResources().getConfiguration().orientation==android.content.res.Configuration.ORIENTATION_LANDSCAPE;int cap=Math.round(getResources().getDisplayMetrics().heightPixels*(landscape?.78f:.60f));super.onMeasure(width,MeasureSpec.makeMeasureSpec(Math.min(desired,cap),MeasureSpec.EXACTLY));}
    void stopRepeating(){for(ImageButton b:new ImageButton[]{left,right,backspace})if(b instanceof RepeatKeyButton)((RepeatKeyButton)b).stopRepeating();}
    void availability(String reason,boolean idle){if(idle&&reason!=null){primary.setEnabled(false);primary.setAlpha(.4f);status.setText(reason);}}
    void mode(String label,boolean busy){mode.setText(label+" · 변경");mode.setEnabled(!busy);mode.setAlpha(busy?.5f:1);mode.setContentDescription(label+" · 처리 방식 변경");}
    void message(String s){status.setText(s);}
    void render(String raw,String polished,boolean recording,boolean processing,boolean password){
        original=raw;corrected=polished;boolean result=!raw.isEmpty()||!polished.isEmpty();boolean busy=recording||processing;
        if(expanded!=result){expanded=result;requestLayout();}
        if(!result)showOriginal=false;if(polished.isEmpty()&&!raw.isEmpty())showOriginal=true;
        results.setVisibility(result?VISIBLE:GONE);idle.setVisibility(result?GONE:VISIBLE);level.setVisibility(recording?VISIBLE:GONE);level.recording(recording);displayedSeconds=-1;
        badge.setText(recording?"듣는 중":processing?"처리 중":result?"교정 완료":"준비");
        idleText.setText(password?"비밀번호 입력칸":recording?"잠시 쉬어도 괜찮아요":processing?"문장을 처리하고 있어요":"목소리를 글로 바꿔보세요");
        primary.setText(recording?"종료":processing?"처리 취소":result?"다시 녹음":"말하기");primary.setContentDescription(primary.getText());primary.setEnabled(!password);primary.setAlpha(password?.4f:1);AppUi.style(primary,!result);
        for(ImageButton b:new ImageButton[]{left,right,backspace,settings}){b.setEnabled(!busy);b.setAlpha(busy?.35f:1);}
        allowInsertion=result&&!busy&&!password;insert.setVisibility(result?VISIBLE:GONE);originalChoice.setVisibility(result?VISIBLE:GONE);originalChoice.setEnabled(allowInsertion&&!raw.isEmpty()&&!polished.isEmpty());
        clear.setText(busy?"취소":"비우기");clear.setVisibility(result||busy?VISIBLE:INVISIBLE);clear.setContentDescription(busy?"현재 작업 취소":"결과 지우기");
        if(result)renderText();
    }
    private void renderText(){rawCard.setSelected(showOriginal);correctedCard.setSelected(!showOriginal);rawCard.setContentDescription("인식 원문 · "+(showOriginal?"선택됨":"눌러서 선택")+" · "+original);correctedCard.setContentDescription("교정문 · "+(!showOriginal?"선택됨":"눌러서 선택")+" · "+corrected);rawView.setText(original);correctedView.setText(corrected.isEmpty()?"교정문 없이 원문을 사용합니다.":corrected);rawCard.setBackground(AppUi.shape(getContext(),showOriginal?AppUi.SELECTED:AppUi.SURFACE,14));correctedCard.setBackground(AppUi.shape(getContext(),showOriginal?AppUi.SURFACE:AppUi.SELECTED,14));originalChoice.setText(showOriginal?"교정문 사용":"원문 사용");originalChoice.setSelected(showOriginal);insert.setText(showOriginal?"원문 삽입":"문장 삽입");insert.setEnabled(allowInsertion&&!(showOriginal?original:corrected).isEmpty());}
    void recordingTime(long seconds){if(displayedSeconds==seconds)return;displayedSeconds=seconds;badge.setText(String.format(java.util.Locale.ROOT,"%d:%02d",seconds/60,seconds%60));badge.setContentDescription("녹음 "+seconds+"초");}
    void recordingLevel(long seconds,int amplitude){recordingTime(seconds);level.sample(amplitude/12000f);}
    private static final class LevelView extends View {
        final Paint paint=new Paint(Paint.ANTI_ALIAS_FLAG);final LevelEnvelope envelope=new LevelEnvelope();final float[] history=new float[17];long historyAt;boolean active;
        void resetMeter(){envelope.reset();java.util.Arrays.fill(history,0);historyAt=0;}
        void recording(boolean next){if(active!=next){active=next;resetMeter();invalidate();}}
        void sample(float value){if(active){envelope.sample(value,android.os.SystemClock.uptimeMillis());invalidate();}}
        @Override protected void onAttachedToWindow(){super.onAttachedToWindow();resetMeter();invalidate();}
        @Override protected void onDetachedFromWindow(){resetMeter();super.onDetachedFromWindow();}
        @Override protected void onWindowVisibilityChanged(int visibility){super.onWindowVisibilityChanged(visibility);if(visibility==VISIBLE)invalidate();}
        LevelView(Context c){super(c);setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);}
        @Override protected void onDraw(Canvas canvas){
            super.onDraw(canvas);long now=android.os.SystemClock.uptimeMillis();float value=envelope.frame(now);
            if(historyAt==0)historyAt=now;
            if(now-historyAt>=60){int steps=(int)Math.min(history.length,(now-historyAt)/60);System.arraycopy(history,steps,history,0,history.length-steps);java.util.Arrays.fill(history,history.length-steps,history.length,0);historyAt=now;}
            history[history.length-1]=Math.max(history[history.length-1],value);
            float gap=Ui.dp(getContext(),5),w=Ui.dp(getContext(),4),middle=getWidth()/2f;
            if(active&&isAttachedToWindow()&&isShown()&&getWindowVisibility()==VISIBLE)postInvalidateOnAnimation();
            paint.setColor(BLUE);
            for(int i=0;i<17;i++){
                float h=Ui.dp(getContext(),3)+(getHeight()-Ui.dp(getContext(),4))*history[i];
                float x=middle+(i-8)*(gap+w)-w/2;
                canvas.drawRoundRect(x,(getHeight()-h)/2,x+w,(getHeight()+h)/2,w,w,paint);
            }
        }
    }
}
