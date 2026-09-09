#import <Foundation/Foundation.h>
#ifndef PROBE_BUILD
#define PROBE_BUILD 1
#endif
static NSDictionary *call(NSString *helper, NSString *op) {
    NSTask *t=[NSTask new]; t.executableURL=[NSURL fileURLWithPath:helper];
    NSPipe *in=[NSPipe pipe], *out=[NSPipe pipe]; t.standardInput=in; t.standardOutput=out;
    NSError *error=nil; if(![t launchAndReturnError:&error]) return @{@"error":@"launch failed"};
    NSDictionary *req=@{@"op":op,@"account":@"llm.h-helper-probe.api_key",@"value":@"non-secret-install-probe"};
    [[in fileHandleForWriting] writeData:[NSJSONSerialization dataWithJSONObject:req options:0 error:nil]];
    [[in fileHandleForWriting] closeFile];
    NSData *data=[[out fileHandleForReading] readDataToEndOfFile]; [t waitUntilExit];
    return [NSJSONSerialization JSONObjectWithData:data options:0 error:nil] ?: @{@"error":@"bad response"};
}
int main(int argc, char **argv) { @autoreleasepool {
    if(argc!=2) return 2; NSString *path=[NSString stringWithUTF8String:argv[1]];
    if(PROBE_BUILD==1 && call(path,@"write")[@"error"]) { puts("write failed"); return 1; }
    NSDictionary *r=call(path,@"read");
    BOOL ok=[r[@"value"] isEqual:@"non-secret-install-probe"];
    printf("build=%d read=%s\n",PROBE_BUILD,ok?"ok":"failed");
    if(PROBE_BUILD==2 && call(path,@"delete")[@"error"]) return 1;
    return ok?0:1;
} }
