using System.Windows.Threading;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static bool State = true;

        public static bool Condition = false;

        public static DispatcherTimer InitializeTimer = new();
    }
}