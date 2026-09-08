package dev.hoilryu.hopentypeless.mobile;
import android.content.Context;
import android.graphics.Color;
import android.widget.*;
final class Ui {
    static int dp(Context c, int n) { return Math.round(n*c.getResources().getDisplayMetrics().density); }
    static LinearLayout column(Context c) {
        LinearLayout layout = new LinearLayout(c); layout.setOrientation(LinearLayout.VERTICAL);
        int p=dp(c,16);layout.setPadding(p,p,p,p);layout.setBackgroundColor(Color.rgb(242,246,252));return layout;
    }
    static TextView text(Context c, LinearLayout parent, String value, int size) {
        TextView view = new TextView(c);view.setText(value);view.setTextSize(size);view.setTextColor(Color.rgb(22,40,62));
        view.setPadding(0,dp(c,6),0,dp(c,6));parent.addView(view);return view;
    }
    static Button button(Context c, LinearLayout parent, String title, Runnable action) {
        Button button = new Button(c);button.setText(title);button.setAllCaps(false);button.setMinHeight(dp(c,48));
        parent.addView(button,new LinearLayout.LayoutParams(-1,-2));button.setOnClickListener(v->action.run());return button;
    }
}
