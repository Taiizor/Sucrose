using SSEET = Sucrose.Space.Enum.EngineType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Engine.Shared.Helper
{
    internal static class Run
    {
        public static bool Check()
        {
            int Result = 0;

            foreach (KeyValuePair<SSEET, string> Pair in SSMI.EngineLive)
            {
                if (SSHC.Work(Pair.Value))
                {
                    Result += SSHC.WorkCount(SSMI.EngineLive[Pair.Key]);
                }
            }

            return Result <= 1;
        }
    }
}