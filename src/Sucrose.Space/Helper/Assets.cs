using System.IO;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Space.Helper
{
    internal static class Assets
    {
        public static string Get(string Assets)
        {
            return Path.Combine(SSMI.This, Assets);
        }
    }
}