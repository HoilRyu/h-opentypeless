#import <Foundation/Foundation.h>
#import <Security/Security.h>
#import <CommonCrypto/CommonDigest.h>
#import <unistd.h>

// One request over inherited anonymous pipes. No listening socket, CLI secrets or logs.
static BOOL trustedParent(void) {
    pid_t parent = getppid();
    if (parent <= 1) return NO;
    SecCodeRef selfCode = NULL, caller = NULL;
    CFDictionaryRef info = NULL;
    SecRequirementRef requirement = NULL;
    BOOL allowed = NO;
    if (SecCodeCopySelf(kSecCSDefaultFlags, &selfCode)) goto done;
    if (SecCodeCopySigningInformation(selfCode, kSecCSSigningInformation, &info)) goto done;
    {
        NSArray *certs = ((__bridge NSDictionary *)info)[(__bridge id)kSecCodeInfoCertificates];
        if (!certs.count) goto done;
        NSData *der = CFBridgingRelease(SecCertificateCopyData((__bridge SecCertificateRef)certs[0]));
        unsigned char digest[CC_SHA1_DIGEST_LENGTH];
        CC_SHA1(der.bytes, (CC_LONG)der.length, digest);
        NSMutableString *hash = [NSMutableString string];
        for (int i = 0; i < CC_SHA1_DIGEST_LENGTH; i++) [hash appendFormat:@"%02x", digest[i]];
        NSString *rule = [NSString stringWithFormat:@"identifier \"dev.hoilryu.hopentypeless\" and certificate leaf = H\"%@\"", hash];
        if (SecRequirementCreateWithString((__bridge CFStringRef)rule, kSecCSDefaultFlags, &requirement)) goto done;
        NSDictionary *guest = @{(__bridge id)kSecGuestAttributePid: @(parent)};
        if (SecCodeCopyGuestWithAttributes(NULL, (__bridge CFDictionaryRef)guest, kSecCSDefaultFlags, &caller)) goto done;
        allowed = SecCodeCheckValidity(caller, kSecCSStrictValidate, requirement) == errSecSuccess && getppid() == parent;
    }
 done:
    if (selfCode) CFRelease(selfCode);
    if (caller) CFRelease(caller);
    if (info) CFRelease(info);
    if (requirement) CFRelease(requirement);
    return allowed;
}
static void reply(NSDictionary *response) {
    NSData *data = [NSJSONSerialization dataWithJSONObject:response options:0 error:nil];
    if (data) fwrite(data.bytes, 1, data.length, stdout);
}
int main(void) { @autoreleasepool {
    if (!trustedParent()) { reply(@{@"error": @"untrusted caller"}); return 1; }
    // Bound request and key sizes. Read to EOF, which the caller closes after writing.
    unsigned char buffer[65537]; size_t count = fread(buffer, 1, sizeof(buffer), stdin);
    if (count > 65536 || ferror(stdin)) { reply(@{@"error": @"invalid request size"}); return 1; }
    id request = [NSJSONSerialization JSONObjectWithData:[NSData dataWithBytes:buffer length:count] options:0 error:nil];
    if (![request isKindOfClass:[NSDictionary class]]) { reply(@{@"error": @"invalid request"}); return 1; }
    NSString *account = request[@"account"], *op = request[@"op"];
    if (![account isKindOfClass:[NSString class]] || account.length > 160 ||
        [account rangeOfString:@"^(stt|llm|session)\\.[a-z0-9._-]+\\.api_key$" options:NSRegularExpressionSearch].location == NSNotFound) {
        reply(@{@"error": @"invalid account"}); return 1;
    }
    NSMutableDictionary *query = [@{(__bridge id)kSecClass:(__bridge id)kSecClassGenericPassword,
        (__bridge id)kSecAttrService:@"H-OpenTypeless", (__bridge id)kSecAttrAccount:account} mutableCopy];
    OSStatus status;
    if ([op isEqual:@"read"]) {
        query[(__bridge id)kSecReturnData] = @YES;
        query[(__bridge id)kSecMatchLimit] = (__bridge id)kSecMatchLimitOne;
        CFTypeRef value = NULL;
        status = SecItemCopyMatching((__bridge CFDictionaryRef)query, &value);
        if (status == errSecItemNotFound) { reply(@{@"value": [NSNull null]}); return 0; }
        if (status == errSecSuccess) {
            NSData *data = CFBridgingRelease(value);
            NSString *text = data.length <= 16384 ? [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding] : nil;
            if (!text) { reply(@{@"error": @"invalid credential encoding or size"}); return 1; }
            reply(@{@"value": text}); return 0;
        }
    } else if ([op isEqual:@"write"]) {
        id text = request[@"value"];
        if (![text isKindOfClass:[NSString class]]) { reply(@{@"error": @"missing value"}); return 1; }
        NSData *data = [text dataUsingEncoding:NSUTF8StringEncoding];
        if (data.length > 16384) { reply(@{@"error": @"credential too large"}); return 1; }
        status = SecItemUpdate((__bridge CFDictionaryRef)query, (__bridge CFDictionaryRef)@{(__bridge id)kSecValueData:data});
        if (status == errSecItemNotFound) {
            query[(__bridge id)kSecValueData] = data;
            query[(__bridge id)kSecAttrLabel] = @"H-OpenTypeless";
            status = SecItemAdd((__bridge CFDictionaryRef)query, NULL);
        }
    } else if ([op isEqual:@"delete"]) {
        status = SecItemDelete((__bridge CFDictionaryRef)query);
        if (status == errSecItemNotFound) status = errSecSuccess;
    } else { reply(@{@"error": @"invalid operation"}); return 1; }
    if (status) { reply(@{@"error": [NSString stringWithFormat:@"Keychain operation failed (%d)", (int)status]}); return 1; }
    reply(@{@"value": [NSNull null]});
    return 0;
} }
