package dev.hoilryu.hopentypeless.mobile;

import org.junit.Test;
import static org.junit.Assert.*;
import java.io.*;
import java.nio.file.Files;

public class PcmAudioTest {
    @Test public void passesHRecordingPcmWithoutTranscoding() throws Exception {
        File input=File.createTempFile("h-pcm-test", ".wav"), output=File.createTempFile("h-pcm-test", ".pcm");
        try {
            byte[] pcm={0,0,-1,127,0,-128};
            try(OutputStream out=new FileOutputStream(input)){out.write(Wav.header(pcm.length));out.write(pcm);}
            PcmAudio.decode(input,output);
            assertArrayEquals(pcm,Files.readAllBytes(output.toPath()));
        } finally {input.delete();output.delete();}
    }
    @Test public void refusesWrongRateAndTruncatedData() throws Exception {
        File input=File.createTempFile("h-pcm-test", ".wav"), output=File.createTempFile("h-pcm-test", ".pcm");
        try {
            for(boolean truncated:new boolean[]{false,true}){
                byte[] header=Wav.header(8);if(!truncated)header[24]=0;
                try(OutputStream out=new FileOutputStream(input)){out.write(header);out.write(new byte[truncated?6:8]);}
                try{PcmAudio.decode(input,output);fail("invalid WAV accepted");}catch(IOException expected){}
            }
        } finally {input.delete();output.delete();}
    }
}
