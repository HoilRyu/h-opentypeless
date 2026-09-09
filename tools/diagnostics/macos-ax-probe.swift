import AppKit
final class Delegate: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var page = 0
    func applicationDidFinishLaunching(_ note: Notification) {
        window = NSWindow(contentRect: NSRect(x: 100,y: 100,width: 480,height: 240), styleMask: [.titled,.closable,.miniaturizable], backing: .buffered,defer: false)
        window.title = "H AX Probe — no audio or network"
        window.isReleasedWhenClosed = false
        render()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
    @objc func nextPage() { page += 1; render() }
    @objc func automaticChanges() {
        var remaining = 200
        Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { [weak self] timer in
            self?.nextPage()
            remaining -= 1
            if remaining == 100 || remaining == 0 {
                let directory = Bundle.main.object(forInfoDictionaryKey: "HProbeDirectory") as! String
                let path = directory + "/phase.txt"
                try? String(200 - remaining).write(toFile: path, atomically: true, encoding: .utf8)
            }
            if remaining == 100 { timer.fireDate = Date(timeIntervalSinceNow: 15) }
            if remaining == 0 { timer.invalidate() }
        }
    }
    func render() {
        let view = NSView(frame: NSRect(x:0,y:0,width:480,height:240))
        let label = NSTextField(labelWithString:"Diagnostic page \(page). No microphone, network, or user data.")
        label.frame = NSRect(x:20,y:155,width:440,height:40)
        let button = NSButton(title:"Replace view",target:self,action:#selector(nextPage))
        button.frame = NSRect(x:20,y:90,width:150,height:36)
        let auto = NSButton(title:"Run two batches",target:self,action:#selector(automaticChanges))
        auto.frame = NSRect(x:200,y:90,width:180,height:36)
        view.addSubview(label); view.addSubview(button); view.addSubview(auto)
        window.contentView = view
    }
}
let app = NSApplication.shared
let delegate = Delegate()
app.delegate = delegate
app.setActivationPolicy(.regular)
app.run()
