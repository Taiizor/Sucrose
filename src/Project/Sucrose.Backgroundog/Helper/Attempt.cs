using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SSSHL = Sucrose.Shared.Space.Helper.Live;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Attempt
    {
        public static async Task Start()
        {
            int MaxAttempts = 5;
            int IntervalSeconds = 1;

            for (int Attempt = 0; Attempt < MaxAttempts; Attempt++)
            {
                if (SSSHL.Run())
                {
                    return;
                }

                await Task.Delay(TimeSpan.FromSeconds(IntervalSeconds));
            }

            SBMI.Exit = false;
            SBMI.Initialize.Stop();

            await Task.CompletedTask;
        }
    }
}