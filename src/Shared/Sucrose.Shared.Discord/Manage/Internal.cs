using DiscordRPC;
using System.Windows.Threading;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;

namespace Sucrose.Shared.Discord.Manage
{
    internal static class Internal
    {
        public static DiscordRpcClient Client;

        public static DispatcherTimer Timer = new();

        public static DateTime? End = Timestamps.Now.End;

        public static DateTime? Start = Timestamps.Now.Start;

        public static List<string> Statement => new()
        {
            SSRER.GetValue("Discord", "StatementText1"),
            SSRER.GetValue("Discord", "StatementText2"),
            SSRER.GetValue("Discord", "StatementText3"),
            SSRER.GetValue("Discord", "StatementText4"),
            SSRER.GetValue("Discord", "StatementText5"),
            SSRER.GetValue("Discord", "StatementText6"),
            SSRER.GetValue("Discord", "StatementText7"),
            SSRER.GetValue("Discord", "StatementText8"),
            SSRER.GetValue("Discord", "StatementText9"),
            SSRER.GetValue("Discord", "StatementText10"),
            SSRER.GetValue("Discord", "StatementText11"),
            SSRER.GetValue("Discord", "StatementText12")
        };

        public static List<string> LargestText => new()
        {
            SSRER.GetValue("Discord", "LargestText1"),
            SSRER.GetValue("Discord", "LargestText2"),
            SSRER.GetValue("Discord", "LargestText3"),
            SSRER.GetValue("Discord", "LargestText4"),
            SSRER.GetValue("Discord", "LargestText5"),
            SSRER.GetValue("Discord", "LargestText6")
        };

        public static List<string> SmallestText => new()
        {
            SSRER.GetValue("Discord", "SmallestText1"),
            SSRER.GetValue("Discord", "SmallestText2"),
            SSRER.GetValue("Discord", "SmallestText3"),
            SSRER.GetValue("Discord", "SmallestText4"),
            SSRER.GetValue("Discord", "SmallestText5"),
            SSRER.GetValue("Discord", "SmallestText6")
        };

        public static readonly List<string> Name = new()
        {
            "Discord.exe",
            "DiscordPTB.exe"
        };

        public static List<string> SmallestImage => new()
        {
            SSRER.GetValue("Discord", "SmallestImage1"),
            SSRER.GetValue("Discord", "SmallestImage2"),
            SSRER.GetValue("Discord", "SmallestImage3"),
            SSRER.GetValue("Discord", "SmallestImage4"),
            SSRER.GetValue("Discord", "SmallestImage5"),
            SSRER.GetValue("Discord", "SmallestImage6"),
            SSRER.GetValue("Discord", "SmallestImage7"),
            SSRER.GetValue("Discord", "SmallestImage8"),
            SSRER.GetValue("Discord", "SmallestImage9"),
            SSRER.GetValue("Discord", "SmallestImage10"),
            SSRER.GetValue("Discord", "SmallestImage11"),
            SSRER.GetValue("Discord", "SmallestImage12"),
            SSRER.GetValue("Discord", "SmallestImage13"),
            SSRER.GetValue("Discord", "SmallestImage14"),
            SSRER.GetValue("Discord", "SmallestImage15"),
            SSRER.GetValue("Discord", "SmallestImage16"),
            SSRER.GetValue("Discord", "SmallestImage17"),
            SSRER.GetValue("Discord", "SmallestImage18"),
            SSRER.GetValue("Discord", "SmallestImage19"),
            SSRER.GetValue("Discord", "SmallestImage20"),
            SSRER.GetValue("Discord", "SmallestImage21"),
            SSRER.GetValue("Discord", "SmallestImage22"),
            SSRER.GetValue("Discord", "SmallestImage23"),
            SSRER.GetValue("Discord", "SmallestImage24"),
            SSRER.GetValue("Discord", "SmallestImage25"),
            SSRER.GetValue("Discord", "SmallestImage26"),
            SSRER.GetValue("Discord", "SmallestImage27"),
            SSRER.GetValue("Discord", "SmallestImage28"),
            SSRER.GetValue("Discord", "SmallestImage29"),
            SSRER.GetValue("Discord", "SmallestImage30"),
            SSRER.GetValue("Discord", "SmallestImage31"),
            SSRER.GetValue("Discord", "SmallestImage32"),
            SSRER.GetValue("Discord", "SmallestImage33"),
            SSRER.GetValue("Discord", "SmallestImage34"),
            SSRER.GetValue("Discord", "SmallestImage35"),
            SSRER.GetValue("Discord", "SmallestImage36")
        };
    }
}