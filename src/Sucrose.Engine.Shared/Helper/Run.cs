using SSEET = Sucrose.Space.Enum.EngineType;
using SSHP = Sucrose.Space.Helper.Processor;
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
                if (SSHP.Work(Pair.Value))
                {
                    Result += SSHP.WorkCount(SSMI.EngineLive[Pair.Key]);
                }
            }

            return Result <= 1;
        }
    }
}