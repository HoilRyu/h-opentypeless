package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import java.io.*;
import java.net.*;
import java.security.MessageDigest;

/** One pinned model at a time. Only verified files are published to the inference engine. */
final class ModelTransfer {
    interface Progress { void update(String text); }
    private volatile boolean canceled;
    private volatile HttpURLConnection connection;
    void cancel(){canceled=true;HttpURLConnection c=connection;if(c!=null)c.disconnect();}
    private void check() throws IOException {if(canceled||Thread.currentThread().isInterrupted())throw new IOException("중단했습니다. 다운로드는 이어받을 수 있습니다.");}
    void download(Context context,ModelCatalog.Model model,Progress progress) throws Exception {
        File target=model.path(context),part=new File(target+".part");
        if(part.length()>model.bytes&&!part.delete())throw new IOException("임시 파일 정리 실패");
        long offset=part.length();
        if(target.getParentFile().getUsableSpace()<model.bytes-offset+64*1024*1024)throw new IOException("저장 공간이 부족합니다.");
        if(offset<model.bytes){
            HttpURLConnection c=open(model.url,offset);connection=c;
            try {
                check();int code=c.getResponseCode();
                if(code==200)offset=0;
                else if(code!=206)throw new IOException("모델 다운로드 HTTP "+code);
                if(code==206&&!validRange(c.getHeaderField("Content-Range"),offset,model.bytes))throw new IOException("이어받기 범위가 일치하지 않습니다.");
                try(InputStream in=c.getInputStream();OutputStream out=new FileOutputStream(part,offset>0)) {copy(in,out,offset,model.bytes,progress);}
            }finally{c.disconnect();connection=null;}
        }
        publish(model,part,target,progress);
    }
    private HttpURLConnection open(String url,long offset) throws Exception {
        for(int i=0;i<6;i++){
            check();URL u=new URL(url);if(!u.getProtocol().equals("https"))throw new IOException("모델은 HTTPS로만 받습니다.");
            HttpURLConnection c=(HttpURLConnection)u.openConnection();connection=c;c.setConnectTimeout(15000);c.setReadTimeout(30000);c.setInstanceFollowRedirects(false);c.setRequestProperty("Accept-Encoding","identity");
            if(offset>0)c.setRequestProperty("Range","bytes="+offset+"-");
            int code=c.getResponseCode();if(code!=301&&code!=302&&code!=303&&code!=307&&code!=308)return c;
            String location=c.getHeaderField("Location");c.disconnect();if(location==null)throw new IOException("다운로드 주소 오류");url=new URL(u,location).toString();
        }throw new IOException("다운로드 리다이렉트가 너무 많습니다.");
    }
    static boolean validRange(String header,long start,long total){
        return header!=null&&header.equals("bytes "+start+"-"+(total-1)+"/"+total);
    }
    void importFile(Context context,ModelCatalog.Model model,InputStream in,Progress progress) throws Exception {
        File target=model.path(context),part=new File(target+".import");
        try {
            if(target.getParentFile().getUsableSpace()<model.bytes+64*1024*1024)throw new IOException("저장 공간이 부족합니다.");
            try(OutputStream out=new FileOutputStream(part)){copy(in,out,0,model.bytes,progress);}
            publish(model,part,target,progress);
        }finally{part.delete();}
    }
    private void copy(InputStream in,OutputStream out,long count,long total,Progress progress) throws Exception {
        byte[] buffer=new byte[256*1024];int n;long last=0;
        while((n=in.read(buffer))!=-1){check();count+=n;if(count>total)throw new IOException("모델 파일 크기가 일치하지 않습니다.");out.write(buffer,0,n);
            long now=System.currentTimeMillis();if(now-last>400){progress.update(String.format(java.util.Locale.ROOT,"다운로드/복사 %.0f%% · %.2f / %.2f GB",100.0*count/total,count/1e9,total/1e9));last=now;}}
        if(count!=total)throw new IOException("전송이 중단되었습니다. 다시 눌러 이어받으세요.");
    }
    private void publish(ModelCatalog.Model model,File part,File target,Progress progress) throws Exception {
        check();if(part.length()!=model.bytes)throw new IOException("모델 크기 불일치");progress.update("파일 무결성 확인 중…");
        MessageDigest hash=MessageDigest.getInstance("SHA-256");
        try(InputStream in=new FileInputStream(part)){byte[] b=new byte[1024*1024];int n;while((n=in.read(b))!=-1){check();hash.update(b,0,n);}}
        StringBuilder hex=new StringBuilder();for(byte b:hash.digest())hex.append(String.format(java.util.Locale.ROOT,"%02x",b&255));
        if(!model.sha256.contentEquals(hex)){part.delete();throw new IOException("모델 체크섬 불일치 · 선택한 모델의 지정 배포 파일을 사용하세요.");}
        check();if(!part.renameTo(target))throw new IOException("모델 설치 마무리 실패");
    }
}
