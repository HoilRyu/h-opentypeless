package dev.hoilryu.hopentypeless.mobile;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.provider.Settings;
import android.text.InputType;
import android.view.inputmethod.InputMethodManager;
import android.widget.*;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class SettingsActivity extends Activity {
    private EditText server;
    private TextView status;
    private Button test;
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private Api.Call call;
    private static final int BG=0xff14171b, CARD=0xff1f242a, INK=0xfff2f5f7, MUTED=0xffa4aeb9, MINT=0xffb4e7d4;
    private ScrollView scroll;
    private int dp(int value){return Ui.dp(this,value);}
    private android.graphics.drawable.GradientDrawable shape(int color,int radius){
        android.graphics.drawable.GradientDrawable d=new android.graphics.drawable.GradientDrawable();
        d.setColor(color);d.setCornerRadius(dp(radius));return d;
    }
    private TextView label(LinearLayout parent,String value,int size,int color){
        TextView v=new TextView(this);v.setText(value);v.setTextSize(size);v.setTextColor(color);
        v.setLineSpacing(dp(3),1);parent.addView(v,new LinearLayout.LayoutParams(-1,-2));return v;
    }
    private LinearLayout section(LinearLayout parent,String title,String subtitle){
        LinearLayout card=new LinearLayout(this);card.setOrientation(LinearLayout.VERTICAL);
        card.setPadding(dp(20),dp(20),dp(20),dp(20));card.setBackground(shape(CARD,24));
        LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,-2);lp.topMargin=dp(16);parent.addView(card,lp);
        TextView heading=label(card,title,19,INK);heading.setTypeface(android.graphics.Typeface.DEFAULT,android.graphics.Typeface.BOLD);
        TextView desc=label(card,subtitle,14,MUTED);desc.setPadding(0,dp(6),0,dp(14));return card;
    }
    private Button action(LinearLayout parent,String title,boolean primary,Runnable run){
        Button v=new Button(this);v.setText(title);v.setAllCaps(false);v.setTextSize(15);
        v.setTextColor(primary?BG:INK);v.setMinHeight(dp(52));v.setPadding(dp(12),dp(10),dp(12),dp(10));
        v.setBackground(new android.graphics.drawable.RippleDrawable(android.content.res.ColorStateList.valueOf(0x337fffff),shape(primary?MINT:0xff30373f,14),null));
        LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,-2);lp.topMargin=dp(10);parent.addView(v,lp);
        v.setOnClickListener(view->run.run());return v;
    }
    private EditText field(LinearLayout parent,String hint){
        EditText v=new EditText(this);v.setTextColor(INK);v.setHintTextColor(MUTED);v.setTextSize(16);
        v.setHint(hint);v.setPadding(dp(14),dp(14),dp(14),dp(14));v.setBackgroundTintList(null);v.setBackground(shape(BG,14));
        parent.addView(v,new LinearLayout.LayoutParams(-1,-2));
        v.setOnFocusChangeListener((view,focused)->{if(focused)revealInput();});return v;
    }
    private void revealInput(){
        scroll.post(()->{
            android.view.View focused=getCurrentFocus();
            if(!(focused instanceof EditText))return;
            EditText input=(EditText)focused;
            android.graphics.Rect rect=new android.graphics.Rect();
            android.text.Layout textLayout=input.getLayout();
            if(textLayout!=null){
                int line=textLayout.getLineForOffset(Math.max(0,input.getSelectionEnd()));
                int y=input.getTotalPaddingTop()-input.getScrollY();
                rect.set(0,y+textLayout.getLineTop(line)-dp(12),input.getWidth(),y+textLayout.getLineBottom(line)+dp(24));
            }else input.getDrawingRect(rect);
            input.requestRectangleOnScreen(rect,false);
        });
    }
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().setSoftInputMode(android.view.WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);
        if(android.os.Build.VERSION.SDK_INT>=30){
            getWindow().setDecorFitsSystemWindows(false);
        }else{
            getWindow().getDecorView().setSystemUiVisibility(0);
            getWindow().setStatusBarColor(BG);getWindow().setNavigationBarColor(BG);
        }
        scroll=new ScrollView(this);scroll.setBackgroundColor(BG);scroll.setFillViewport(true);
        scroll.setClipToPadding(false);
        LinearLayout layout=new LinearLayout(this);layout.setOrientation(LinearLayout.VERTICAL);
        layout.setPadding(dp(20),dp(20),dp(20),dp(28));layout.setFocusableInTouchMode(true);
        scroll.addView(layout);setContentView(scroll);
        // PhoneWindow needs its decor view before an insets controller can be queried.
        if(android.os.Build.VERSION.SDK_INT>=30){
            android.view.WindowInsetsController controller=getWindow().getInsetsController();
            if(controller!=null)controller.setSystemBarsAppearance(0,
                android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS|android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS);
        }
        if(android.os.Build.VERSION.SDK_INT>=30){
            scroll.setOnApplyWindowInsetsListener((view,insets)->{
                android.graphics.Insets bars=insets.getInsets(android.view.WindowInsets.Type.systemBars());
                int keyboard=insets.getInsets(android.view.WindowInsets.Type.ime()).bottom;
                view.setPadding(bars.left,bars.top,bars.right,Math.max(bars.bottom,keyboard));
                revealInput();return android.view.WindowInsets.CONSUMED;
            });
            scroll.requestApplyInsets();
        }
        scroll.addOnLayoutChangeListener((v,l,t,r,b,ol,ot,or,ob)->{if(b-t!=ob-ot)revealInput();});
        TextView brand=label(layout,"H-OpenTypeless",30,INK);brand.setTypeface(android.graphics.Typeface.DEFAULT,android.graphics.Typeface.BOLD);
        TextView intro=label(layout,"말로 쓰는 나만의 키보드",15,MUTED);intro.setPadding(0,dp(6),0,dp(8));

        LinearLayout connection=section(layout,"H 앱 연결","같은 Wi-Fi 또는 WireGuard에서 연결하세요.");
        TextView addressLabel=label(connection,"서버 주소",13,MUTED);addressLabel.setPadding(0,0,0,dp(8));
        server=field(connection,"http://192.168.0.123:8787");server.setId(R.id.server_address);server.setSingleLine(true);
        server.setInputType(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_VARIATION_URI);server.setText(Config.server(this));
        action(connection,"연결 설정 저장",false,()->save());
        test=action(connection,"연결 확인",true,()->test());
        status=label(connection,"서버 주소를 저장하고 연결을 확인하세요.",13,MUTED);status.setPadding(0,dp(14),0,0);
        status.setAccessibilityLiveRegion(android.view.View.ACCESSIBILITY_LIVE_REGION_POLITE);

        LinearLayout setup=section(layout,"키보드 준비","처음 한 번만 설정하면 됩니다.");
        action(setup,"마이크 권한 허용",false,()->{
            if(checkSelfPermission(Manifest.permission.RECORD_AUDIO)==PackageManager.PERMISSION_GRANTED)status.setText("마이크 권한이 허용되어 있습니다.");
            else requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO},1);
        });
        action(setup,"키보드 활성화",false,()->startActivity(new Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)));
        action(setup,"사용할 키보드 선택",false,()->((InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showInputMethodPicker());

        LinearLayout trial=section(layout,"바로 써보기","입력칸을 누르고 녹음한 뒤, 결과를 삽입해 보세요.");
        EditText sample=field(trial,"여기에 음성으로 입력해 보세요");sample.setId(R.id.sample_input);
        sample.setInputType(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_FLAG_MULTI_LINE|InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
        sample.setGravity(android.view.Gravity.TOP);sample.setMinLines(3);sample.setMaxLines(6);
        sample.setImeOptions(android.view.inputmethod.EditorInfo.IME_FLAG_NO_EXTRACT_UI);
    }
    private boolean save() {
        try { Config.save(this,server.getText().toString());status.setText("연결 설정을 저장했습니다.");return true; }
        catch(Exception e){status.setText(e instanceof IllegalArgumentException?e.getMessage():"설정을 저장하지 못했습니다. 다시 시도해 주세요.");return false;}
    }
    private void test() {
        if(!save())return;
        String address=server.getText().toString();
        test.setEnabled(false);status.setText(R.string.connecting);call=new Api.Call();Api.Call current=call;
        worker.execute(()->{
            String message;
            try { org.json.JSONObject health=current.run(address,null);
                if(!"h-opentypeless".equals(health.optString("service"))||health.optInt("protocol")!=1)throw new java.io.IOException("H-OpenTypeless의 모바일 연결 주소인지 확인하세요.");
                message="H 앱에 연결했습니다. 음성 모델은 녹음으로 확인해 주세요."; }
            catch(Exception e){message=Api.message(e);}
            String result=message;runOnUiThread(()->{if(!isDestroyed()){status.setText(result);test.setEnabled(true);}});
        });
    }
    @Override public void onRequestPermissionsResult(int code,String[] permissions,int[] grants){
        super.onRequestPermissionsResult(code,permissions,grants);
        if(code==1)status.setText(grants.length>0&&grants[0]==PackageManager.PERMISSION_GRANTED?"마이크 권한을 허용했습니다.":"마이크 권한이 필요합니다. 거부가 계속되면 Android 앱 설정에서 허용해 주세요.");
    }
    @Override public void onDestroy(){if(call!=null)call.cancel();worker.shutdownNow();super.onDestroy();}
}
