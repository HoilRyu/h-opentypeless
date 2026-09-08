package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.graphics.drawable.RippleDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.*;

/** Compact IME surface: only the transcript scrolls, never the primary actions. */
final class KeyboardPanel extends LinearLayout {
    interface Actions { void record(); void insert(String value); void clear(); void keyboard(); void settings(); void moveLeft(); void moveRight(); void backspace(); }
    private static final int BG=Color.rgb(20,23,27), SURFACE=Color.rgb(31,36,42), INK=Color.rgb(242,245,247), MUTED=Color.rgb(164,174,185), BLUE=Color.rgb(180,231,212), RED=Color.rgb(255,181,177);
    private final TextView badge, status, preview, recordLabel;
    private final Button insert, correctedTab, originalTab, clear;
    private final ImageButton primary;
    private final LinearLayout tabs;
    private final LevelView level;
    private final ImageButton left, right, backspace;
    private String original="",corrected="";
    private boolean showOriginal, allowInsertion, expanded;
    private int bottomInset;

    KeyboardPanel(Context context,Actions actions) {
        super(context);setOrientation(VERTICAL);setBackgroundColor(BG);
        int p=dp(16);bottomInset=dp(28);setPadding(p,dp(8),p,dp(10)+bottomInset);
        setOnApplyWindowInsetsListener((view,insets)->{
            int navigation;
            if(android.os.Build.VERSION.SDK_INT>=30) navigation=insets.getInsets(android.view.WindowInsets.Type.navigationBars()).bottom;
            else navigation=insets.getSystemWindowInsetBottom();
            int safe=Math.max(dp(24),navigation);
            if(safe!=bottomInset){bottomInset=safe;setPadding(p,dp(8),p,dp(10)+bottomInset);requestLayout();}
            return insets;
        });
        LinearLayout header=row();
        TextView brand=text("H-OpenTypeless",16,INK);brand.setTypeface(Typeface.create("sans-serif-medium",Typeface.NORMAL));
        header.addView(brand,new LayoutParams(0,-1,1));
        badge=text("준비",13,MUTED);badge.setGravity(Gravity.CENTER);header.addView(badge,new LayoutParams(dp(60),-1));
        header.addView(icon(KeyIcon.Kind.KEYBOARD,"다른 키보드로 전환",actions::keyboard,false),new LayoutParams(dp(48),dp(48)));
        header.addView(icon(KeyIcon.Kind.SETTINGS,"연결 설정",actions::settings,false),new LayoutParams(dp(48),dp(48)));
        addView(header,new LayoutParams(-1,dp(48)));

        LinearLayout card=new LinearLayout(context);card.setOrientation(VERTICAL);card.setPadding(dp(16),dp(4),dp(16),dp(8));card.setBackground(shape(SURFACE,20,false));
        LayoutParams cardParams=new LayoutParams(-1,0,1);cardParams.topMargin=dp(6);addView(card,cardParams);
        tabs=row();
        correctedTab=button("교정문",()->{showOriginal=false;renderText();});
        originalTab=button("원문",()->{showOriginal=true;renderText();});
        tabs.addView(correctedTab,new LayoutParams(dp(76),dp(48)));tabs.addView(originalTab,new LayoutParams(dp(64),dp(48)));
        View spacer=new View(context);tabs.addView(spacer,new LayoutParams(0,1,1));
        clear=button("비우기",actions::clear);tabs.addView(clear,new LayoutParams(dp(64),dp(48)));
        card.addView(tabs,new LayoutParams(-1,dp(48)));
        LinearLayout content=row();
        ScrollView scroll=new ScrollView(context);scroll.setFillViewport(true);scroll.setVerticalScrollBarEnabled(true);
        preview=text("녹음을 시작하세요",17,INK);preview.setGravity(Gravity.CENTER_VERTICAL);preview.setLineSpacing(dp(4),1);preview.setPadding(0,dp(4),dp(8),dp(4));
        scroll.addView(preview,new ScrollView.LayoutParams(-1,-1));content.addView(scroll,new LayoutParams(0,-1,1));
        LinearLayout recordingAction=new LinearLayout(context);recordingAction.setOrientation(VERTICAL);recordingAction.setGravity(Gravity.CENTER);
        primary=icon(KeyIcon.Kind.MICROPHONE,"녹음 시작",actions::record,true);
        recordingAction.addView(primary,new LayoutParams(dp(56),dp(56)));
        recordLabel=text("녹음",12,MUTED);recordLabel.setGravity(Gravity.CENTER);recordingAction.addView(recordLabel,new LayoutParams(-1,dp(24)));
        content.addView(recordingAction,new LayoutParams(dp(64),-1));card.addView(content,new LayoutParams(-1,0,1));
        level=new LevelView(context);card.addView(level,new LayoutParams(-1,dp(20)));level.setVisibility(GONE);

        status=text("최대 2분 · 확인 후 직접 삽입",14,MUTED);status.setMaxLines(2);status.setGravity(Gravity.CENTER_VERTICAL);
        status.setAccessibilityLiveRegion(View.ACCESSIBILITY_LIVE_REGION_POLITE);
        addView(status,new LayoutParams(-1,dp(40)));
        LinearLayout editing=row();
        editing.setPadding(dp(4),0,dp(4),0);editing.setBackground(shape(SURFACE,14,false));
        left=icon(KeyIcon.Kind.LEFT,"커서 왼쪽으로 이동",actions::moveLeft,false);
        right=icon(KeyIcon.Kind.RIGHT,"커서 오른쪽으로 이동",actions::moveRight,false);
        backspace=icon(KeyIcon.Kind.BACKSPACE,"Backspace · 앞 글자 또는 선택 영역 삭제",actions::backspace,false);
        editing.addView(left,new LayoutParams(0,dp(48),1));editing.addView(right,new LayoutParams(0,dp(48),1));
        editing.addView(backspace,new LayoutParams(0,dp(48),1));
        insert=button("교정문 삽입",()->actions.insert(showOriginal?original:corrected));insert.setTextSize(14);insert.setTypeface(Typeface.DEFAULT_BOLD);
        LayoutParams insertParams=new LayoutParams(0,dp(48),1);editing.addView(insert,insertParams);
        addView(editing,new LayoutParams(-1,dp(48)));
        render("","",false,false,false);
    }
    private int dp(int value){return Ui.dp(getContext(),value);}
    private LinearLayout row(){LinearLayout row=new LinearLayout(getContext());row.setGravity(Gravity.CENTER_VERTICAL);return row;}
    private TextView text(String value,int size,int color){TextView v=new TextView(getContext());v.setText(value);v.setTextSize(size);v.setTextColor(color);v.setIncludeFontPadding(false);v.setGravity(Gravity.CENTER_VERTICAL);return v;}
    private GradientDrawable shape(int color,int radius,boolean stroke){GradientDrawable d=new GradientDrawable();d.setColor(color);d.setCornerRadius(dp(radius));if(stroke)d.setStroke(dp(1),Color.rgb(49,64,83));return d;}
    private void style(Button button,int background,int foreground){button.setTextColor(foreground);button.setBackground(new RippleDrawable(ColorStateList.valueOf(0x33B4E7D4),shape(background,16,false),null));}
    private Button button(String title,Runnable action){Button b=new Button(getContext());b.setText(title);b.setAllCaps(false);b.setTextSize(14);b.setTypeface(Typeface.create("sans-serif-medium",Typeface.NORMAL));b.setIncludeFontPadding(false);b.setMinWidth(0);b.setMinimumWidth(0);b.setMinHeight(0);b.setMinimumHeight(0);b.setPadding(dp(4),0,dp(4),0);style(b,Color.TRANSPARENT,MUTED);b.setOnClickListener(v->action.run());return b;}
    private ImageButton icon(KeyIcon.Kind kind,String label,Runnable action,boolean filled){
        boolean repeats=kind==KeyIcon.Kind.LEFT||kind==KeyIcon.Kind.RIGHT||kind==KeyIcon.Kind.BACKSPACE;
        ImageButton b=repeats?new RepeatKeyButton(getContext()):new ImageButton(getContext());b.setImageDrawable(new KeyIcon(kind,INK));
        b.setScaleType(ImageView.ScaleType.FIT_CENTER);b.setPadding(dp(14),dp(12),dp(14),dp(12));
        b.setContentDescription(label+(repeats?", 길게 누르면 반복":""));if(!repeats)b.setTooltipText(label);
        b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x33B4E7D4),shape(filled?SURFACE:Color.TRANSPARENT,14,false),null));
        b.setOnClickListener(v->{v.performHapticFeedback(android.view.HapticFeedbackConstants.KEYBOARD_TAP);action.run();});return b;
    }
    @Override protected void onMeasure(int width,int height){
        float scale=Math.max(1,Math.min(1.6f,getResources().getConfiguration().fontScale));
        int desired=Math.round(dp(expanded?304:256)*scale)+bottomInset;
        boolean landscape=getResources().getConfiguration().orientation==android.content.res.Configuration.ORIENTATION_LANDSCAPE;
        int maximum=Math.round(getResources().getDisplayMetrics().heightPixels*(landscape?.75f:.60f));
        super.onMeasure(width,MeasureSpec.makeMeasureSpec(Math.min(desired,maximum),MeasureSpec.EXACTLY));
    }
    void stopRepeating(){
        if(left instanceof RepeatKeyButton)((RepeatKeyButton)left).stopRepeating();
        if(right instanceof RepeatKeyButton)((RepeatKeyButton)right).stopRepeating();
        if(backspace instanceof RepeatKeyButton)((RepeatKeyButton)backspace).stopRepeating();
    }
    void message(String message){status.setText(message);}
    void render(String raw,String polished,boolean recording,boolean processing,boolean password){
        original=raw;corrected=polished;
        boolean result=!raw.isEmpty()||!polished.isEmpty();
        boolean more=result||recording;
        if(expanded!=more){expanded=more;requestLayout();}
        if(!result)showOriginal=false;
        if(polished.isEmpty()&&!raw.isEmpty())showOriginal=true;
        tabs.setVisibility(result||recording?VISIBLE:GONE);
        correctedTab.setVisibility(result?VISIBLE:GONE);originalTab.setVisibility(result?VISIBLE:GONE);
        clear.setText(recording?"취소":"비우기");
        clear.setContentDescription(recording?"현재 녹음 취소":"받아쓰기 결과 비우기");
        left.setEnabled(!recording&&!processing);right.setEnabled(!recording&&!processing);backspace.setEnabled(!recording&&!processing);
        left.setAlpha(recording||processing?.35f:1);right.setAlpha(left.getAlpha());backspace.setAlpha(left.getAlpha());
        level.setVisibility(recording?VISIBLE:GONE);
        badge.setContentDescription(null);
        badge.setText(recording?"녹음":processing?"처리 중":result?"완료":"준비");badge.setTextColor(recording?RED:BLUE);
        primary.setImageDrawable(new KeyIcon(recording?KeyIcon.Kind.STOP:processing?KeyIcon.Kind.CLOSE:KeyIcon.Kind.MICROPHONE,BG));
        primary.setContentDescription(recording?"녹음 종료 · 보내기":processing?"처리 취소":result?"다시 녹음":"녹음 시작");
        primary.setTooltipText(primary.getContentDescription());recordLabel.setVisibility(recording?GONE:VISIBLE);recordLabel.setText(recording?"종료":processing?"취소":result?"다시":"녹음");
        // During processing the primary action becomes cancellation.
        primary.setEnabled(!password);
        primary.setAlpha(password?.4f:1);
        allowInsertion=result&&!password&&!recording&&!processing;
        insert.setVisibility(result?VISIBLE:GONE);insert.setEnabled(allowInsertion);
        primary.setBackground(new RippleDrawable(ColorStateList.valueOf(0x33FFFFFF),shape(recording?RED:BLUE,22,false),null));
        style(insert,BLUE,BG);
        if(result)renderText();
        else {preview.setText(password?"비밀번호 입력칸":recording?"듣고 있습니다…":processing?"문장을 다듬고 있습니다…":"녹음을 시작하세요");preview.setTextColor(recording?INK:MUTED);}
    }
    private void renderText(){
        preview.setText(showOriginal?original:corrected);preview.setTextColor(INK);
        style(correctedTab,showOriginal?Color.TRANSPARENT:0xFF354B44,showOriginal?MUTED:BLUE);
        style(originalTab,showOriginal?0xFF354B44:Color.TRANSPARENT,showOriginal?BLUE:MUTED);
        correctedTab.setSelected(!showOriginal);originalTab.setSelected(showOriginal);
        insert.setText(showOriginal?"원문 삽입":"교정문 삽입");
        insert.setEnabled(allowInsertion&&!(showOriginal?original:corrected).isEmpty());
    }
    void recordingLevel(long seconds,int amplitude){
        badge.setText(String.format(java.util.Locale.ROOT,"%d:%02d",seconds/60,seconds%60));
        badge.setContentDescription("녹음 "+seconds+"초");level.value=Math.min(1f,amplitude/12000f);level.invalidate();
    }
    private static final class LevelView extends View {
        final Paint paint=new Paint(Paint.ANTI_ALIAS_FLAG);float value;
        LevelView(Context c){super(c);setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);}
        @Override protected void onDraw(Canvas canvas){
            super.onDraw(canvas);float gap=Ui.dp(getContext(),5),w=Ui.dp(getContext(),4),middle=getWidth()/2f;
            paint.setColor(BLUE);
            for(int i=0;i<17;i++){
                float envelope=1f-Math.abs(i-8)/10f;
                float h=Ui.dp(getContext(),3)+(getHeight()-Ui.dp(getContext(),4))*value*envelope;
                float x=middle+(i-8)*(gap+w)-w/2;
                canvas.drawRoundRect(x,(getHeight()-h)/2,x+w,(getHeight()+h)/2,w,w,paint);
            }
        }
    }
}
