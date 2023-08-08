using DiscordRPC;
using System.Windows.Threading;

namespace Sucrose.Shared.Discord.Manage
{
    internal static class Internal
    {
        public static DiscordRpcClient Client;

        public static DateTime? End = Timestamps.Now.End;

        public static DispatcherTimer RefreshTimer = new();

        public static DateTime? Start = Timestamps.Now.Start;

        public static DispatcherTimer InitializeTimer = new();

        public static readonly List<string> Name = new()
        {
            "Discord.exe",
            "DiscordPTB.exe"
        };
    }
}