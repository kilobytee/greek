import SwiftUI

@main
struct GreekApp: App {
    @StateObject private var model = GreekAppModel()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(model)
                .preferredColorScheme(.dark)
        }
    }
}
