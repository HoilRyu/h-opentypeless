package dev.hoilryu.hopentypeless.mobile;

import android.app.Activity;
import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Typeface;
import android.graphics.drawable.*;
import android.view.*;
import android.widget.*;

/** Desktop H design tokens, adapted to Android touch sizes. */
final class AppUi {
    static final int BG=0xff1a1a1a,SURFACE=0xff242424,ELEVATED=0xff2a2a2a,INK=0xfff0f0f0,MUTED=0xffa0a0a0,ACCENT=0xff2abba7,SELECTED=0xff1a3d37,BORDER=0xff383838;
    static int dp(Context c,int n){return Ui.dp(c,n);}
    static GradientDrawable shape(Context c,int color,int radius){GradientDrawable d=new GradientDrawable();d.setColor(color);d.setCornerRadius(dp(c,radius));d.setStroke(dp(c,1),color==ACCENT?0xff48cbbb:BORDER);return d;}
    static LinearLayout column(Context c){LinearLayout l=new LinearLayout(c);l.setOrientation(LinearLayout.VERTICAL);return l;}
    static TextView text(LinearLayout p,String value,int size,int color){Context c=p.getContext();TextView t=new TextView(c);t.setText(value);t.setTextSize(size);t.setTextColor(color);t.setLineSpacing(dp(c,3),1);t.setPadding(0,dp(c,5),0,dp(c,5));p.addView(t,new LinearLayout.LayoutParams(-1,-2));return t;}
    static TextView title(LinearLayout p,String value){TextView t=text(p,value,24,INK);t.setTypeface(Typeface.DEFAULT,Typeface.BOLD);return t;}
    static LinearLayout card(LinearLayout p,String title,String body){Context c=p.getContext();LinearLayout l=column(c);int v=dp(c,20);l.setPadding(v,v,v,v);l.setBackground(shape(c,SURFACE,14));LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,-2);lp.topMargin=dp(c,24);p.addView(l,lp);if(!title.isEmpty()){TextView t=text(l,title,18,INK);t.setTypeface(Typeface.DEFAULT,Typeface.BOLD);}if(!body.isEmpty())text(l,body,14,MUTED);return l;}
    static Button button(LinearLayout p,String label,boolean primary,Runnable action){Context c=p.getContext();Button b=new Button(c);b.setText(label);b.setTextSize(15);b.setAllCaps(false);b.setMinHeight(dp(c,52));b.setMinimumWidth(0);b.setMinWidth(0);b.setPadding(dp(c,16),dp(c,12),dp(c,16),dp(c,12));style(b,primary);LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,-2);lp.topMargin=dp(c,12);p.addView(b,lp);b.setOnClickListener(v->action.run());return b;}
    static void style(Button b,boolean primary){b.setTextColor(primary?BG:INK);b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x332abba7),shape(b.getContext(),primary?ACCENT:ELEVATED,12),null));}
    static void tab(Button b,boolean selected){b.setTextColor(selected?ACCENT:MUTED);b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x332abba7),shape(b.getContext(),selected?SELECTED:BG,12),null));b.setSelected(selected);}
    static LinearLayout.LayoutParams cell(Context c,int index){LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(0,-2,1);if(index>0)lp.leftMargin=dp(c,8);return lp;}
    static Button row(LinearLayout p,KeyIcon.Kind icon,String title,String body,Runnable action){Button b=button(p,title+(body.isEmpty()?"":"\n"+body),false,action);b.setGravity(Gravity.START|Gravity.CENTER_VERTICAL);b.setTextSize(15);b.setLineSpacing(dp(p.getContext(),4),1);if(!body.isEmpty()){android.text.SpannableString content=new android.text.SpannableString(title+"\n"+body);int start=title.length()+1;content.setSpan(new android.text.style.ForegroundColorSpan(MUTED),start,content.length(),0);content.setSpan(new android.text.style.RelativeSizeSpan(.87f),start,content.length(),0);b.setText(content);}KeyIcon d=new KeyIcon(icon,MUTED);d.setBounds(0,0,dp(b.getContext(),24),dp(b.getContext(),24));b.setCompoundDrawables(d,null,null,null);b.setCompoundDrawablePadding(dp(b.getContext(),14));return b;}
    static EditText field(LinearLayout p,String hint){Context c=p.getContext();EditText e=new EditText(c);e.setTextSize(16);e.setTextColor(INK);e.setHintTextColor(MUTED);e.setHint(hint);e.setPadding(dp(c,14),dp(c,12),dp(c,14),dp(c,12));e.setBackground(shape(c,BG,12));LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,-2);lp.topMargin=dp(c,10);p.addView(e,lp);return e;}
    static void screen(Activity a,View root){a.getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);a.setContentView(root);root.setBackgroundColor(BG);if(android.os.Build.VERSION.SDK_INT>=30){a.getWindow().setDecorFitsSystemWindows(false);a.getWindow().getInsetsController().setSystemBarsAppearance(0,WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS|WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS);root.setOnApplyWindowInsetsListener((v,i)->{android.graphics.Insets bars=i.getInsets(WindowInsets.Type.systemBars());v.setPadding(bars.left,bars.top,bars.right,Math.max(bars.bottom,i.getInsets(WindowInsets.Type.ime()).bottom));return WindowInsets.CONSUMED;});root.requestApplyInsets();}else{a.getWindow().setStatusBarColor(BG);a.getWindow().setNavigationBarColor(BG);}}
}
