package dev.hoilryu.hopentypeless.mobile;

import android.graphics.*;
import android.graphics.drawable.Drawable;

/** Consistent 24-unit stroke icons, independent of the device's text font. */
final class KeyIcon extends Drawable {
    enum Kind { LEFT, RIGHT, BACKSPACE, KEYBOARD, SETTINGS, MICROPHONE, STOP, CLOSE, HOME, HELP, MODEL, PHONE, COMPUTER, SPARK }
    private final Kind kind;
    private final Paint paint=new Paint(Paint.ANTI_ALIAS_FLAG);
    KeyIcon(Kind kind,int color){this.kind=kind;paint.setColor(color);paint.setStyle(Paint.Style.STROKE);paint.setStrokeWidth(1.7f);paint.setStrokeCap(Paint.Cap.ROUND);paint.setStrokeJoin(Paint.Join.ROUND);}
    private void line(Canvas c,float... points){Path p=new Path();p.moveTo(points[0],points[1]);for(int i=2;i<points.length;i+=2)p.lineTo(points[i],points[i+1]);c.drawPath(p,paint);}
    @Override public void draw(Canvas c){
        Rect b=getBounds();c.save();c.translate(b.left,b.top);c.scale(b.width()/24f,b.height()/24f);
        switch(kind){
            case HOME:line(c,3,11,12,3,21,11);line(c,5,10,5,21,10,21,10,15,14,15,14,21,19,21,19,10);break;
            case HELP:c.drawCircle(12,12,9,paint);c.drawArc(9,6,15,12,190,300,false,paint);line(c,12,12,12,14);line(c,12,18,12,18.1f);break;
            case MODEL:line(c,3,7,12,2,21,7,21,17,12,22,3,17,3,7,12,12,21,7);line(c,12,12,12,22);break;
            case PHONE:c.drawRoundRect(6,2,18,22,2,2,paint);line(c,10,5,14,5);line(c,11,19,13,19);break;
            case COMPUTER:c.drawRoundRect(3,3,21,16,2,2,paint);line(c,12,16,12,21);line(c,7,21,17,21);break;
            case SPARK:line(c,12,2,15,9,22,12,15,15,12,22,9,15,2,12,9,9,12,2);break;
            case LEFT:line(c,14,6,8,12,14,18);break;
            case RIGHT:line(c,10,6,16,12,10,18);break;
            case BACKSPACE:line(c,9,5,21,5,21,19,9,19,2,12,9,5);line(c,12,9,17,15);line(c,17,9,12,15);break;
            case KEYBOARD:
                c.drawRoundRect(2,5,22,19,3,3,paint);
                for(int x=6;x<=18;x+=4){line(c,x,9,x+.1f,9);line(c,x,12,x+.1f,12);}line(c,7,16,17,16);break;
            case SETTINGS:
                c.drawCircle(12,12,6,paint);c.drawCircle(12,12,2.2f,paint);
                for(int i=0;i<8;i++){c.save();c.rotate(i*45,12,12);line(c,12,3,12,5);c.restore();}break;
            case STOP:c.drawRoundRect(6,6,18,18,2,2,paint);break;
            case CLOSE:line(c,6,6,18,18);line(c,18,6,6,18);break;
            case MICROPHONE:
                c.drawRoundRect(9,3,15,14,3,3,paint);c.drawArc(6,6,18,18,0,180,false,paint);line(c,12,18,12,21);line(c,9,21,15,21);break;
        }
        c.restore();
    }
    @Override public int getIntrinsicWidth(){return 24;}
    @Override public int getIntrinsicHeight(){return 24;}
    @Override public void setAlpha(int alpha){paint.setAlpha(alpha);invalidateSelf();}
    @Override public void setColorFilter(ColorFilter filter){paint.setColorFilter(filter);invalidateSelf();}
    @Override public int getOpacity(){return PixelFormat.TRANSLUCENT;}
}
