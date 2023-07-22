using DiscordRPC;
using System.Windows.Threading;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;

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
            SGHDL.GetValue("StatementText1"),
            SGHDL.GetValue("StatementText2"),
            SGHDL.GetValue("StatementText3"),
            SGHDL.GetValue("StatementText4"),
            SGHDL.GetValue("StatementText5"),
            SGHDL.GetValue("StatementText6"),
            SGHDL.GetValue("StatementText7"),
            SGHDL.GetValue("StatementText8"),
            SGHDL.GetValue("StatementText9"),
            SGHDL.GetValue("StatementText10"),
            SGHDL.GetValue("StatementText11"),
            SGHDL.GetValue("StatementText12")
        };

        public static List<string> LargestText => new()
        {
            SGHDL.GetValue("LargestText1"),
            SGHDL.GetValue("LargestText2"),
            SGHDL.GetValue("LargestText3"),
            SGHDL.GetValue("LargestText4"),
            SGHDL.GetValue("LargestText5"),
            SGHDL.GetValue("LargestText6")
        };

        public static List<string> SmallestText => new()
        {
            SGHDL.GetValue("SmallestText1"),
            SGHDL.GetValue("SmallestText2"),
            SGHDL.GetValue("SmallestText3"),
            SGHDL.GetValue("SmallestText4"),
            SGHDL.GetValue("SmallestText5"),
            SGHDL.GetValue("SmallestText6")
        };

        public static readonly List<string> Name = new()
        {
            "Discord.exe",
            "DiscordPTB.exe"
        };

        public static List<string> SmallestImage => new()
        {
            SGHDL.GetValue("SmallestImage1"),
            SGHDL.GetValue("SmallestImage2"),
            SGHDL.GetValue("SmallestImage3"),
            SGHDL.GetValue("SmallestImage4"),
            SGHDL.GetValue("SmallestImage5"),
            SGHDL.GetValue("SmallestImage6"),
            SGHDL.GetValue("SmallestImage7"),
            SGHDL.GetValue("SmallestImage8"),
            SGHDL.GetValue("SmallestImage9"),
            SGHDL.GetValue("SmallestImage10"),
            SGHDL.GetValue("SmallestImage11"),
            SGHDL.GetValue("SmallestImage12"),
            SGHDL.GetValue("SmallestImage13"),
            SGHDL.GetValue("SmallestImage14"),
            SGHDL.GetValue("SmallestImage15"),
            SGHDL.GetValue("SmallestImage16"),
            SGHDL.GetValue("SmallestImage17"),
            SGHDL.GetValue("SmallestImage18"),
            SGHDL.GetValue("SmallestImage19"),
            SGHDL.GetValue("SmallestImage20"),
            SGHDL.GetValue("SmallestImage21"),
            SGHDL.GetValue("SmallestImage22"),
            SGHDL.GetValue("SmallestImage23"),
            SGHDL.GetValue("SmallestImage24"),
            SGHDL.GetValue("SmallestImage25"),
            SGHDL.GetValue("SmallestImage26"),
            SGHDL.GetValue("SmallestImage27"),
            SGHDL.GetValue("SmallestImage28"),
            SGHDL.GetValue("SmallestImage29"),
            SGHDL.GetValue("SmallestImage30"),
            SGHDL.GetValue("SmallestImage31"),
            SGHDL.GetValue("SmallestImage32"),
            SGHDL.GetValue("SmallestImage33"),
            SGHDL.GetValue("SmallestImage34"),
            SGHDL.GetValue("SmallestImage35"),
            SGHDL.GetValue("SmallestImage36")
        };
    }
}