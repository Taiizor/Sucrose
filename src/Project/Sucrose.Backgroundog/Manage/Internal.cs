using Timer = System.Threading.Timer;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static bool State = true;

        public static bool Condition = false;

        public static bool Processing = true;

        public static Timer InitializeTimer = null;
    }
}