using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRGH = Sucrose.Memory.Manage.Readonly.GitHub;
using SMMRS = Sucrose.Memory.Manage.Readonly.Soferity;

namespace Sucrose.Memory.Manage.Readonly
{
    public static class Url
    {
        public static readonly string GitHub = "https://github.com";

        public static readonly string Local = $"https://{SMMRG.Local}/";

        public static readonly string OfficialWebPage = "https://sucrose.dev";

        public static readonly string Soferity = "https://sucrose.soferity.com";

        public static readonly string PremiumStore = $"{OfficialWebPage}/premium";

        public static readonly string RawGitHub = "https://raw.githubusercontent.com";

        public static readonly string VCRedist = "https://aka.ms/vc14/vc_redist.{0}.exe";

        public static readonly string GoogleSearch = "https://www.google.com/search?q={0}";

        public static readonly string SoferityStore = $"{Soferity}/{SMMRS.StoreDirectory}";

        public static readonly string WebView2 = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

        public static readonly string FFmpeg = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";

        public static readonly string GitHubStore = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}";

        public static readonly string GitHubSucrose = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}";

        public static readonly string YouTubePersonalAccessToken = "https://www.youtube.com/watch?v=kRyML8axJxA";

        public static readonly string GitHubStoreWiki = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}/{SMMRGH.Wiki}";

        public static readonly string GitHubSucroseWiki = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Wiki}";

        public static readonly string GoogleTranslate = "https://translate.google.com/?sl=auto&tl=auto&text={0}&op=translate";

        public static readonly string RawGitHubStoreBranch = $"{RawGitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}/{SMMRGH.Branch}";

        public static readonly string GitHubSucroseRelease = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Releases}";

        public static readonly string MicrosoftStoreSucrose = "https://apps.microsoft.com/detail/XP8JGPBHTJGLCQ?launch=true&mode=full";

        public static readonly string GitHubStoreDiscussions = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}/{SMMRGH.Discussions}";

        public static readonly string RawGitHubSucroseBranch = $"{RawGitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Branch}";

        public static readonly string GitHubSucroseDiscussions = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Discussions}";

        public static readonly string GitHubSucroseReleaseTag = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Releases}/{SMMRGH.Tag}";

        public static readonly string GitHubSucroseReleaseLatest = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Releases}/{SMMRGH.Latest}";

        public static readonly string GitHubStoreReport = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}/{SMMRGH.Issues}/{SMMRGH.New}/{SMMRGH.Choose}";

        public static readonly string GitHubSucroseReport = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.SucroseRepository}/{SMMRGH.Issues}/{SMMRGH.New}/{SMMRGH.Choose}";

        public static readonly string GitHubStoreReportWallpaper = $"{GitHub}/{SMMRGH.Owner}/{SMMRGH.StoreRepository}/{SMMRGH.Issues}/{SMMRGH.New}?assignees=&labels=wallpaper&projects=&template=wallpaper_report.yaml";
    }
}