using SDEET = Sucrose.Dependency.Enum.EngineType;
using SSHP = Sucrose.Space.Helper.Processor;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Space.Helper
{
    internal static class Live
    {
        public static bool Run()
        {
            foreach (KeyValuePair<SDEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHP.Work(Pair.Value))
                {
                    return true;
                }
            }

            return false;
        }

        public static bool Run(SDEET Live)
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
            foreach (KeyValuePair<SDEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHP.Work(Pair.Value))
                {
                    SSHP.Kill(Pair.Value);
                }
            }
        }

        public static void Kill(SDEET Live)
        {
            SSHP.Kill(SSMI.EngineLive[Live]);
        }

        public static void Kill(string Live)
        {
            SSHP.Kill(SSMI.TextEngineLive[Live]);
        }
    }
}