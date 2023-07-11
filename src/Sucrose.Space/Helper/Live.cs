using SSEET = Sucrose.Space.Enum.EngineType;
using SSHP = Sucrose.Space.Helper.Processor;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Space.Helper
{
    internal static class Live
    {
        public static bool Run()
        {
            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHP.Work(Pair.Value))
                {
                    return true;
                }
            }

            return false;
        }

        public static bool Run(SSEET Live)
        {
            if (SSHP.Work(SSMI.EngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static bool Run(string Live)
        {
            if (SSHP.Work(SSMI.TextEngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static void Kill()
        {
            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHP.Work(Pair.Value))
                {
                    SSHP.Kill(Pair.Value);
                }
            }
        }

        public static void Kill(SSEET Live)
        {
            SSHP.Kill(SSMI.EngineLive[Live]);
        }

        public static void Kill(string Live)
        {
            SSHP.Kill(SSMI.TextEngineLive[Live]);
        }
    }
}