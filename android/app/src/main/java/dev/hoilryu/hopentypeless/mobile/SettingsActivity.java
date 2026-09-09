package dev.hoilryu.hopentypeless.mobile;

import android.Manifest;
import android.app.*;
import android.content.*;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.provider.Settings;
import android.text.InputType;
import android.view.*;
import android.view.inputmethod.InputMethodManager;
import android.widget.*;
import java.util.concurrent.*;

public final class SettingsActivity extends Activity {
    private String page="home",draftAddress;private boolean displayedLocal,refreshOnResume;
    private EditText server;private TextView status;private Button test;
    private final ExecutorService worker=Executors.newSingleThreadExecutor();private Api.Call call;private int generation;
    @Override public void onCreate(Bundle state){super.onCreate(state);if(state!=null){page=state.getString("page","home");draftAddress=state.getString("address");}build();}
    @Override public void onSaveInstanceState(Bundle state){state.putString("page",page);state.putString("address",server==null?draftAddress:server.getText().toString());super.onSaveInstanceState(state);}
    @Override public void onResume(){super.onResume();if(refreshOnResume||displayedLocal!=LocalConfig.local(this)){refreshOnResume=false;build();}}
    private void build(){
        if(server!=null)draftAddress=server.getText().toString();server=null;generation++;if(call!=null){call.cancel();call=null;}
        displayedLocal=LocalConfig.local(this);
        LinearLayout shell=AppUi.column(this);ScrollView scroll=new ScrollView(this);scroll.setFillViewport(true);
        LinearLayout root=AppUi.column(this);root.setPadding(dp(20),dp(16),dp(20),dp(24));scroll.addView(root);shell.addView(scroll,new LinearLayout.LayoutParams(-1,0,1));
        AppUi.title(root,"H-OpenTypeless");AppUi.text(root,"AI 음성 입력",13,AppUi.MUTED);
        status=AppUi.text(root,"",14,AppUi.ACCENT);status.setVisibility(View.GONE);status.setAccessibilityLiveRegion(View.ACCESSIBILITY_LIVE_REGION_POLITE);
        if(page.equals("home"))home(root);else if(page.equals("settings"))settings(root);else help(root);
        LinearLayout nav=new LinearLayout(this);nav.setPadding(dp(20),dp(8),dp(20),dp(8));
        String[] names={"홈","설정","도움말"},pages={"home","settings","help"};KeyIcon.Kind[] icons={KeyIcon.Kind.HOME,KeyIcon.Kind.SETTINGS,KeyIcon.Kind.HELP};
        for(int i=0;i<3;i++){final String next=pages[i];LinearLayout cell=AppUi.column(this);nav.addView(cell,AppUi.cell(this,i));Button b=AppUi.button(cell,names[i],page.equals(next),()->{page=next;build();});AppUi.tab(b,page.equals(next));KeyIcon icon=new KeyIcon(icons[i],page.equals(next)?AppUi.ACCENT:AppUi.MUTED);icon.setBounds(0,0,dp(20),dp(20));b.setCompoundDrawables(null,icon,null,null);b.setTextSize(12);b.setSelected(page.equals(next));}
        shell.addView(nav);AppUi.screen(this,shell);
    }
    private int dp(int n){return AppUi.dp(this,n);}
    private void message(String s){status.setText(s);status.setVisibility(View.VISIBLE);}
    private void chooseMode(){
        new AlertDialog.Builder(this).setTitle("어디에서 처리할까요?").setSingleChoiceItems(new String[]{"이 기기에서 · 로컬\n준비된 모델로 인터넷 없이 사용","내 컴퓨터에서 · 원격\n컴퓨터의 H 앱에 연결"},displayedLocal?0:1,(dialog,which)->{LocalConfig.local(this,which==0);dialog.dismiss();page="home";build();}).setNegativeButton("닫기",null).show();
    }
    private void localPage(String tab){refreshOnResume=true;startActivity(new Intent(this,LocalSettingsActivity.class).putExtra("tab",tab));}
    private String modelLabel(String id){return id.equals("none")?"교정 끄기 · 원문 사용":id.equals("system")?"Android 기본 STT":ModelCatalog.get(this,id).label;}
    private void home(LinearLayout root){
        AppUi.title(root,displayedLocal?"목소리로 쓰는 하루.":"컴퓨터의 성능을, 손안에서.");
        AppUi.button(root,displayedLocal?"이 기기에서 처리 · 변경":"내 컴퓨터에서 처리 · 변경",true,this::chooseMode);
        if(displayedLocal){
            String reason=Capability.selected(this);LinearLayout ready=AppUi.card(root,reason==null?"모델 사용 준비 완료":"사용 전 확인이 필요해요",android.os.Build.MODEL+" · "+(reason==null?"기기 지원 확인됨":reason));
            AppUi.text(ready,"마이크와 키보드 준비는 설정에서 확인하세요.",13,AppUi.MUTED);
            LinearLayout input=AppUi.card(root,"내 음성 입력","");
            AppUi.row(input,KeyIcon.Kind.MICROPHONE,"음성 인식 · "+modelLabel(LocalConfig.stt(this)),LocalConfig.stt(this).equals("system")?"종료 버튼을 누를 때까지 계속 들어요":"녹음 후 글로 바꿔요 · 최대 119초",()->localPage("stt"));
            AppUi.row(input,KeyIcon.Kind.SPARK,"문장 교정 · "+modelLabel(LocalConfig.llm(this)),LocalConfig.llm(this).equals("none")?"인식한 원문을 그대로 사용해요":"질문에 답하지 않고 문장만 다듬어요",()->localPage("llm"));
            AppUi.text(input,"준비된 모델로 기기 안에서 처리해요.",13,AppUi.MUTED);
            AppUi.row(root,KeyIcon.Kind.MODEL,"모델 관리","다운로드 · 가져오기 · 실행 확인",()->localPage("llm"));
        }else connection(root);
        LinearLayout trial=AppUi.card(root,"바로 써보기","입력칸을 누르고 음성 키보드를 열어보세요.");EditText sample=AppUi.field(trial,"여기에 말로 입력해 보세요");sample.setId(R.id.sample_input);sample.setInputType(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_FLAG_MULTI_LINE|InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);sample.setGravity(Gravity.TOP);sample.setMinLines(3);sample.setMaxLines(6);sample.setImeOptions(android.view.inputmethod.EditorInfo.IME_FLAG_NO_EXTRACT_UI);
    }
    private void connection(LinearLayout root){
        LinearLayout card=AppUi.card(root,"컴퓨터 연결","음성 인식과 교정은 컴퓨터에 설정된 모델을 사용해요.");
        AppUi.text(card,"서버 주소",13,AppUi.MUTED);server=AppUi.field(card,"http://192.168.0.123:8787");server.setId(R.id.server_address);server.setSingleLine(true);server.setInputType(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_VARIATION_URI);server.setText(draftAddress==null?Config.server(this):draftAddress);
        AppUi.text(card,"컴퓨터의 H 앱에 표시된 주소를 입력하세요.",13,AppUi.MUTED);AppUi.button(card,"저장",false,this::save);test=AppUi.button(card,"연결 확인",true,this::test);
        AppUi.text(card,"같은 Wi-Fi 또는 WireGuard로 연결하세요.\n컴퓨터에서 H 앱을 실행해 주세요.\n연결이 끊겨도 다른 처리 방식으로 전환하지 않아요.",14,AppUi.MUTED);
    }
    private void settings(LinearLayout root){
        AppUi.title(root,"사용 설정");
        if(displayedLocal){AppUi.row(root,KeyIcon.Kind.SPARK,"내 말투 그대로","교정 모델 · 문체 지침",()->localPage("preferences"));AppUi.row(root,KeyIcon.Kind.MODEL,"모델 관리","설치와 실행 상태 확인",()->localPage("stt"));}
        else AppUi.row(root,KeyIcon.Kind.COMPUTER,"컴퓨터 연결 설정","주소 저장과 연결 확인",()->{page="home";build();});
        LinearLayout setup=AppUi.card(root,"키보드 준비","처음 한 번 준비하면 다른 앱에서도 사용할 수 있어요.");
        AppUi.row(setup,KeyIcon.Kind.MICROPHONE,"마이크 권한",checkSelfPermission(Manifest.permission.RECORD_AUDIO)==PackageManager.PERMISSION_GRANTED?"허용됨":"허용 필요",()->{if(checkSelfPermission(Manifest.permission.RECORD_AUDIO)==PackageManager.PERMISSION_GRANTED)message("마이크 권한이 허용되어 있습니다.");else requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO},1);});
        AppUi.row(setup,KeyIcon.Kind.KEYBOARD,"음성 키보드 활성화","Android 키보드 설정 열기",()->startActivity(new Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)));
        AppUi.row(setup,KeyIcon.Kind.KEYBOARD,"키보드 선택","H-OpenTypeless 음성 키보드 사용",()->((InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showInputMethodPicker());
        AppUi.button(root,"처리 방식 변경",false,this::chooseMode);
    }
    private void help(LinearLayout root){
        AppUi.title(root,"알아두면 좋아요");
        AppUi.card(root,"어떻게 입력하나요?","다른 앱의 입력칸에서 음성 키보드를 열고 말하기를 누르세요. 종료 후 원문과 교정문을 확인하고 삽입하세요. 메시지 전송은 직접 합니다.");
        AppUi.card(root,"녹음은 언제 끝나나요?",displayedLocal?"기본 STT는 종료 버튼을 누를 때까지 계속 듣습니다. 인식기 재시작 사이에는 짧은 공백이 있을 수 있어요. Whisper 녹음은 최대 119초입니다.":"종료 버튼을 누르면 음성을 컴퓨터에 전달해요. 현재 녹음은 최대 119초입니다.");
        if(displayedLocal){AppUi.card(root,"모델을 사용할 수 없어요","기기 지원·메모리·발열·설치 상태를 확인해요. 기본 STT의 한국어 모델은 음성 인식 설정에서 준비하세요. 모델별 사용 조건이 다릅니다.");AppUi.card(root,"마지막 처리 정보",LocalConfig.prefs(this).getString("last-result","아직 처리 정보가 없습니다."));AppUi.card(root,"기본 STT 진단",LocalConfig.prefs(this).getString("system-session-info","아직 기본 STT 진단 정보가 없습니다.")+"\n시작 횟수가 반복해서 늘면 인식기가 세션을 종료하고 다시 시작한 것입니다. 연속 구간은 인식기를 유지한 채 받은 문장 수입니다.");AppUi.text(root,"발화 내용의 기록 목록을 만들지 않습니다. 기본 STT의 결과 대기 시간과 Whisper의 로딩·인식 시간은 직접 비교하지 마세요.",13,AppUi.MUTED);}
        else AppUi.card(root,"연결이 안 돼요","컴퓨터의 H 앱 실행 상태, 주소, 같은 네트워크 또는 WireGuard 연결을 확인하세요. 실패하면 원인을 안내하며 자동으로 처리 방식을 변경하지 않습니다.");
    }
    private boolean save(){try{Config.save(this,server.getText().toString());draftAddress=server.getText().toString();message("연결 설정을 저장했습니다.");return true;}catch(Exception e){message(e instanceof IllegalArgumentException?e.getMessage():"설정을 저장하지 못했습니다.");return false;}}
    private void test(){if(!save())return;String address=server.getText().toString();final int current=generation;test.setEnabled(false);message("연결 확인 중…");call=new Api.Call();Api.Call request=call;worker.execute(()->{String value;try{org.json.JSONObject h=request.run(address,null);if(!"h-opentypeless".equals(h.optString("service"))||h.optInt("protocol")!=1)throw new java.io.IOException("H 앱의 모바일 연결 주소인지 확인하세요.");value="H 앱에 연결됨 · 음성 모델은 녹음으로 확인하세요.";}catch(Exception e){value=Api.message(e);}String result=value;runOnUiThread(()->{if(!isDestroyed()&&current==generation){call=null;message(result);test.setEnabled(true);}});});}
    @Override public void onRequestPermissionsResult(int code,String[] permissions,int[] grants){super.onRequestPermissionsResult(code,permissions,grants);if(code==1){build();message(grants.length>0&&grants[0]==PackageManager.PERMISSION_GRANTED?"마이크 권한을 허용했습니다.":"Android 앱 설정에서 마이크 권한을 허용해 주세요.");}}
    @Override public void onDestroy(){generation++;if(call!=null)call.cancel();worker.shutdownNow();super.onDestroy();}
}
