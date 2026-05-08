cask "lumos" do
  version "0.1.0"
  sha256 :no_check # updated automatically by release workflow

  url "https://github.com/heza-ru/Lumos/releases/download/v#{version}/Lumos_#{version}_universal.dmg"
  name "Lumos"
  desc "Native macOS screen annotation for live demos, presentations, and teaching"
  homepage "https://github.com/heza-ru/Lumos"

  depends_on macos: ">= :ventura" # macOS 13+

  app "Lumos.app"

  zap trash: [
    "~/Library/Application Support/com.lumos.app",
    "~/Library/Preferences/com.lumos.app.plist",
    "~/Library/Saved Application State/com.lumos.app.savedState",
  ]
end
