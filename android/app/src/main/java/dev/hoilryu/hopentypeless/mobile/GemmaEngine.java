package dev.hoilryu.hopentypeless.mobile;

import com.google.ai.edge.litertlm.*;
import java.util.Collections;

final class GemmaEngine {
    static String polish(String path,String cache,String prompt,String raw,android.os.Bundle metrics) throws Exception {
        if(raw.length()>10000||prompt.length()>4000)throw new IllegalArgumentException("교정 입력이 너무 깁니다.");
        EngineConfig config=new EngineConfig(path,new Backend.GPU(),null,null,4096,null,cache);
        long loading=android.os.SystemClock.elapsedRealtime();
        try(Engine engine=new Engine(config)){
            engine.initialize();
            metrics.putLong("llm_load_ms",android.os.SystemClock.elapsedRealtime()-loading);
            ConversationConfig conversationConfig=new ConversationConfig(
                Contents.Companion.of(DictationContract.system(prompt)),Collections.emptyList(),Collections.emptyList(),
                new SamplerConfig(1,0.95,0.0,42),false,Collections.emptyList(),Collections.emptyMap(),null,false,
                1536,new ThinkingConfig(false,0),false);
            try(Conversation conversation=engine.createConversation(conversationConfig)){
                long inference=android.os.SystemClock.elapsedRealtime();
                String result=conversation.sendMessage(DictationContract.message(raw)).toString().trim();
                metrics.putLong("llm_infer_ms",android.os.SystemClock.elapsedRealtime()-inference);
                DictationContract.validate(raw,result);
                return result;
            }
        }
    }
}
