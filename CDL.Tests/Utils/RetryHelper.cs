using System;
using System.Threading;

namespace CDL.Tests.Utils
{
    public class RetryHelper
    {
        public static T TryFetch<T>(Func<T> request, Func<T, bool> validator, int retries = 10, int delayMs = 3000)
        {
            for (var i = 0; i < retries; i++)
            {
                var resp = request.Invoke();
                if (validator.Invoke(resp))
                {
                    return resp;
                }
                Thread.Sleep(delayMs);
            }

            throw new Exception("Exceeded maximum number of retries");
        }
    }
}
