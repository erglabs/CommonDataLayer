using CDL.Tests.Configuration;
using Grpc.Core;
using MaterializerOndemand;
using Microsoft.Extensions.Options;
using System.Collections.Generic;
using System.Threading.Tasks;
using static MaterializerOndemand.OnDemandMaterializer;

namespace CDL.Tests.Services
{
    public class OnDemandMaterializerService
    {
        private ConfigurationOptions _options;
        private OnDemandMaterializerClient _client;

        public OnDemandMaterializerService(IOptions<ConfigurationOptions> options, OnDemandMaterializerClient client)
        {
            _options = options.Value;
            _client = client;
        }

        public Task<Empty> Heartbeat()
        {
            return Task.FromResult(_client.Heartbeat(new Empty()));
        }

        public AsyncServerStreamingCall<Common.RowDefinition> Materialize(string viewId, IList<string> schemaIds)
        {
            var request = new OnDemandRequest(){
                ViewId = viewId
            };
            var schema = new Schema();

            foreach (var item in schemaIds)
            {
                request.Schemas.Add(item, schema);
            }

            
            
            return _client.Materialize(request);
        }
        
    }
}