using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSSHL = Sucrose.Shared.Space.Helper.Live;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Attempt
    {
        public static async Task Start()
        {
            int MaxAttempt = 5;
            int IntervalSecond = 1;

            for (int Attempt = 0; Attempt < MaxAttempt; Attempt++)
            {
                if (SSSHL.Run())
                {
                    await Task.Delay(TimeSpan.FromSeconds(3));

                    return;
                }

                await Task.Delay(TimeSpan.FromSeconds(IntervalSecond));
            }

            SBMI.Exit = false;
            SBMI.Initialize.Stop();

            SMMI.BackgroundogSettingManager.SetSetting(new KeyValuePair<string, bool>[]
            {
                new(SMC.ClosePerformance, false),
                new(SMC.PausePerformance, false)
            });

            await Task.CompletedTask;
        }
    }
}