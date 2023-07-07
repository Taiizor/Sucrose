using SSEET = Sucrose.Space.Enum.EngineType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Space.Helper
{
    internal static class Live
    {
        public static bool Run()
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

        public static bool Run(SSEET Live)
        {
            if (SSHC.Work(SSMI.EngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static bool Run(string Live)
        {
            if (SSHC.Work(SSMI.TextEngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static void Kill()
        {
            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHC.Work(Pair.Value))
                {
                    SSHC.Kill(Pair.Value);
                }
            }
        }

        public static void Kill(SSEET Live)
        {
            SSHC.Kill(SSMI.EngineLive[Live]);
        }

        public static void Kill(string Live)
        {
            SSHC.Kill(SSMI.TextEngineLive[Live]);
        }
    }
}