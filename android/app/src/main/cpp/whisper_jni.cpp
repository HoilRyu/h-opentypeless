#include <jni.h>
#include <whisper.h>
#include <fstream>
#include <vector>
#include <string>
#include <algorithm>
#include <memory>
#include <limits>
#include <chrono>
#include <new>

static void fail(JNIEnv* env, const char* message, const char* type="java/io/IOException") {
    if (env->ExceptionCheck()) return;
    jclass cls=env->FindClass(type);
    if (cls) { env->ThrowNew(cls,message); env->DeleteLocalRef(cls); }
}
struct UtfChars {
    JNIEnv* env; jstring value; const char* chars;
    UtfChars(JNIEnv* e,jstring v):env(e),value(v),chars(v?e->GetStringUTFChars(v,nullptr):nullptr) {}
    ~UtfChars(){if(chars)env->ReleaseStringUTFChars(value,chars);}
};
using Clock=std::chrono::steady_clock;
static jlong elapsed(Clock::time_point start){return std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now()-start).count();}
extern "C" JNIEXPORT jbyteArray JNICALL Java_dev_hoilryu_hopentypeless_mobile_WhisperNative_transcribe(
    JNIEnv* env,jclass,jstring model,jstring pcm,jint threads,jlongArray timings) {
    try {
        if(!model||!pcm){fail(env,"Missing model or PCM path");return nullptr;}
        UtfChars modelPath(env,model);
        if(!modelPath.chars)return nullptr;
        UtfChars pcmPath(env,pcm);
        if(!pcmPath.chars)return nullptr;
        std::ifstream in(pcmPath.chars,std::ios::binary|std::ios::ate);
        auto size=in?static_cast<long long>(in.tellg()):-1;
        if(size<=0||size>16000LL*2*121||size%2){fail(env,"Invalid PCM audio");return nullptr;}
        std::vector<int16_t> samples(size/2);
        in.seekg(0);in.read(reinterpret_cast<char*>(samples.data()),size);
        if(!in){fail(env,"PCM read failed");return nullptr;}
        std::vector<float> audio(samples.size());
        for(size_t i=0;i<samples.size();i++)audio[i]=samples[i]/32768.f;
        auto cp=whisper_context_default_params();cp.use_gpu=false;
        auto start=Clock::now();
        std::unique_ptr<whisper_context,decltype(&whisper_free)> ctx(
            whisper_init_from_file_with_params(modelPath.chars,cp),whisper_free);
        if(!ctx){fail(env,"Whisper model initialization failed");return nullptr;}
        jlong load=elapsed(start);
        auto params=whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
        params.language="ko";params.translate=false;params.no_context=true;
        params.n_threads=std::max(1,std::min(4,(int)threads));
        params.print_progress=false;params.print_realtime=false;params.print_timestamps=false;params.print_special=false;
        start=Clock::now();
        int result=whisper_full(ctx.get(),params,audio.data(),(int)audio.size());
        jlong infer=elapsed(start);
        if(result!=0){fail(env,"Whisper transcription failed");return nullptr;}
        std::string text;
        for(int i=0;i<whisper_full_n_segments(ctx.get());i++){
            const char* part=whisper_full_get_segment_text(ctx.get(),i);
            if(part)text+=part;
        }
        if(text.size()>static_cast<size_t>(std::numeric_limits<jsize>::max())){
            fail(env,"Whisper output too large");return nullptr;
        }
        if(timings&&env->GetArrayLength(timings)>=2){
            jlong values[]={load,infer};env->SetLongArrayRegion(timings,0,2,values);
            if(env->ExceptionCheck())return nullptr;
        }
        auto bytes=env->NewByteArray(static_cast<jsize>(text.size()));
        if(!bytes)return nullptr;
        env->SetByteArrayRegion(bytes,0,static_cast<jsize>(text.size()),reinterpret_cast<const jbyte*>(text.data()));
        return env->ExceptionCheck()?nullptr:bytes;
    } catch(const std::bad_alloc&) {
        fail(env,"Not enough memory for Whisper","java/lang/OutOfMemoryError");
    } catch(const std::exception&) {
        fail(env,"Whisper native processing failed");
    } catch(...) {
        fail(env,"Unexpected Whisper native failure");
    }
    return nullptr;
}
