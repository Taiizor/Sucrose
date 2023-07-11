using DiscordRPC;
using SGHDL = Sucrose.Globalization.Helper.DiscordLocalization;

namespace Sucrose.Discord.Manage
{
    internal static class Internal
    {
        public static DiscordRpcClient Client;

        public static readonly List<string> Name = new()
        {
            "Discord.exe",
            "DiscordPTB.exe"
        };

        public static readonly List<string> Largest = new()
        {
            SGHDL.GetValue("LargestText1"),
            SGHDL.GetValue("LargestText2"),
            SGHDL.GetValue("LargestText3"),
            SGHDL.GetValue("LargestText4"),
            SGHDL.GetValue("LargestText5"),
            SGHDL.GetValue("LargestText6")
        };

        public static readonly List<string> Smallest = new()
        {
            SGHDL.GetValue("SmallestText1"),
            SGHDL.GetValue("SmallestText2"),
            SGHDL.GetValue("SmallestText3"),
            SGHDL.GetValue("SmallestText4"),
            SGHDL.GetValue("SmallestText5"),
            SGHDL.GetValue("SmallestText6")
        };

        public static readonly List<string> Statement = new()
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
    }
}