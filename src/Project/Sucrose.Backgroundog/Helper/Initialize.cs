using SBHA = Sucrose.Backgroundog.Helper.Attempt;
using SBHC = Sucrose.Backgroundog.Helper.Condition;
using SBHP = Sucrose.Backgroundog.Helper.Performance;
using SBHS = Sucrose.Backgroundog.Helper.Specification;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SSSHL = Sucrose.Shared.Space.Helper.Live;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Initialize
    {
        public static void Start()
        {
            SBMI.Computer.Open();
            TimerCallback Callback = InitializeTimer_Callback;
            SBMI.InitializeTimer = new(Callback, null, 0, 1000);
        }

        public static void Stop()
        {
            SBMI.Computer.Close();
            SBMI.InitializeTimer.Dispose();
        }

        private static async void InitializeTimer_Callback(object State)
        {
            _ = SBHS.Start();

            if (SBMI.Processing)
            {
                SBMI.Processing = false;
                Console.WriteLine("Callback");

                if (SSSHL.Run())
                {
                    await SBHP.Start();
                }
                else if (SBMI.Condition)
                {
                    await SBHC.Start();
                }
                else
                {
                    await SBHA.Start();
                }

                SBMI.Processing = true;
            }
        }
    }
}
