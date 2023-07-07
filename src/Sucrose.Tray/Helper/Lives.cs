using SSEET = Sucrose.Space.Enum.EngineType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Tray.Helper
{
    internal static class Lives
    {
        public static bool RunLive()
        {
            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHC.Work(Pair.Value))
                {
                    return true;
                }
            }

            return false;
        }

        public static bool RunLive(SSEET Live)
        {
            if (SSHC.Work(SSMI.EngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static bool RunLive(string Live)
        {
            if (SSHC.Work(SSMI.TextEngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static void KillLive()
        {
            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHC.Work(Pair.Value))
                {
                    SSHC.Kill(Pair.Value);
                }
            }
        }

        public static void KillLive(SSEET Live)
        {
            SSHC.Kill(SSMI.EngineLive[Live]);
        }

        public static void KillLive(string Live)
        {
            SSHC.Kill(SSMI.TextEngineLive[Live]);
        }
    }
}