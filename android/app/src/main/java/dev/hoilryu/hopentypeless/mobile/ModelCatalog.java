package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import org.json.*;
import java.io.*;
import java.util.*;

final class ModelCatalog {
    static final class Model {
        final String id,label,file,url,sha256; final long bytes; final double ramGb,availableGb;
        Model(JSONObject j) throws JSONException {id=j.getString("id");label=j.getString("label");file=j.getString("file");url=j.getString("url");sha256=j.getString("sha256");bytes=j.getLong("bytes");ramGb=j.getDouble("ramGb");availableGb=j.getDouble("availableGb");}
        boolean whisper(){return id.startsWith("whisper-");}
        File path(Context c){File dir=new File(c.getFilesDir(),"models");dir.mkdirs();return new File(dir,file);}
        boolean installed(Context c){return path(c).length()==bytes;}
        String size(){return bytes<1000000000?String.format(Locale.ROOT,"%.0f MB",bytes/1e6):String.format(Locale.ROOT,"%.2f GB",bytes/1e9);}
    }
    static List<Model> all(Context c){
        try(InputStream in=c.getAssets().open("models.json")){
            ByteArrayOutputStream out=new ByteArrayOutputStream();byte[] b=new byte[4096];int n;while((n=in.read(b))!=-1)out.write(b,0,n);
            JSONArray a=new JSONArray(out.toString("UTF-8"));List<Model> result=new ArrayList<>();for(int i=0;i<a.length();i++)result.add(new Model(a.getJSONObject(i)));return result;
        }catch(Exception e){throw new IllegalStateException("모델 목록을 읽지 못했습니다.",e);}
    }
    static Model get(Context c,String id){for(Model m:all(c))if(m.id.equals(id))return m;throw new IllegalArgumentException("지원하지 않는 모델입니다.");}
}
