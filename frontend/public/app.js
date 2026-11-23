const names = {employee: 'Ananya Rao', manager: 'Shreyas', member: 'Rahul'};
const categories = {
  Incident: ['Network Issue', 'VPN Issue', 'Login Issue', 'System Performance', 'Others'],
  'Service Request': ['Software Installation', 'Hardware Request', 'Access Permission', 'Others']
};
const team = [
  {name: 'Rahul', categories: ['Network Issue', 'VPN Issue', 'Login Issue', 'System Performance', 'Others'], active: true},
  {name: 'Priya', categories: ['Software Installation', 'Access Permission', 'Others'], active: true},
  {name: 'Vikram', categories: ['Hardware Request', 'System Performance'], active: true}
];
const tickets = [
  {id:'INC-1042', title:'Office internet keeps disconnecting', category:'Network Issue', priority:'P2', status:'In Progress', owner:'Ananya Rao', assignee:'Rahul', type:'Incident', description:'My office connection drops during video calls. This happens several times every morning.'},
  {id:'SR-1038', title:'Install Figma on my laptop', category:'Software Installation', priority:'P3', status:'Open', owner:'Ananya Rao', assignee:'', type:'Service Request', description:'I need Figma for the new design project.'},
  {id:'INC-1036', title:'Unable to connect to VPN', category:'VPN Issue', priority:'P2', status:'Awaiting Confirmation', owner:'Ananya Rao', assignee:'Rahul', type:'Incident', description:'The VPN client stops at the sign-in screen.', resolution:'Updated the VPN client and refreshed your access.', root:'The installed client version was outdated.'},
  {id:'INC-1031', title:'Laptop running slowly', category:'System Performance', priority:'P4', status:'Resolved', owner:'Ananya Rao', assignee:'Rahul', type:'Incident', description:'Applications take a long time to open.', resolution:'Removed unused startup applications.', root:'Too many background processes.'},
  {id:'INC-1045', title:'Office network unavailable', category:'Network Issue', priority:'P1', status:'Open', owner:'Kiran Kumar', assignee:'', type:'Incident', description:'The entire second floor has lost network access.'},
  {id:'SR-1040', title:'Access to the analytics workspace', category:'Access Permission', priority:'P3', status:'Open', owner:'Meera S', assignee:'', type:'Service Request', description:'Please grant access to the analytics workspace.'}
].map(t => ({...t, messages:[{author:t.owner, text:t.description}], history:['Ticket created'], image:null}));
let role = 'employee';
let page = 'Dashboard';
let selected = null;
let search = '';
let filter = 'All statuses';
const $ = id => document.getElementById(id);
const escapeHtml = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const badge = text => '<span class="badge '+({P1:'p1',P2:'p2',P3:'p3',P4:'p4','In Progress':'progress','Awaiting Confirmation':'awaiting',Resolved:'resolved'}[text] || '')+'">'+escapeHtml(text)+'</span>';
function visibleTickets() {
  return tickets.filter(t => role === 'manager' || (role === 'employee' ? t.owner === names.employee : t.assignee === names.member));
}
function notice(text) {
  $('toast').textContent = text;
  $('toast').style.display = 'block';
  clearTimeout(notice.timer);
  notice.timer = setTimeout(() => $('toast').style.display = 'none', 3500);
}
function render() {
  const nav = role === 'employee' ? ['Dashboard','My tickets','Create ticket'] : role === 'manager' ? ['Dashboard','All tickets','Team members','Performance'] : ['Dashboard','Assigned tickets','In progress','Completed'];
  $('nav').innerHTML = nav.map(n => '<button data-page="'+n+'" class="'+(page===n?'active':'')+'">'+n+'</button>').join('');
  $('role-name').textContent = {employee:'Employee',manager:'IT Manager',member:'IT Team Member'}[role];
  $('title').textContent = page === 'Dashboard' ? {employee:'Your support, simplified.',manager:'Support overview',member:'Let’s get things working.'}[role] : page;
  $('subtitle').textContent = {employee:'Create requests, follow progress, and stay connected with IT.',manager:'Keep the queue moving and your team focused.',member:'Your assigned work and employee conversations, together.'}[role];
  $('new-ticket').hidden = role !== 'employee';
  const rows = visibleTickets();
  const stats = role === 'manager' ? [
    ['Total tickets',rows.length,'Across your workspace'],
    ['Awaiting assignment',rows.filter(t=>!t.assignee).length,'Ready for manager review'],
    ['Critical P1',rows.filter(t=>t.priority==='P1'&&t.status!=='Resolved').length,'Handled by you'],
    ['Resolved',rows.filter(t=>t.status==='Resolved').length,'Confirmed by employees']
  ] : [
    ['Open',rows.filter(t=>t.status==='Open').length,'Ready for the next step'],
    ['In progress',rows.filter(t=>t.status==='In Progress').length,'Being investigated'],
    ['Awaiting confirmation',rows.filter(t=>t.status==='Awaiting Confirmation').length,'Ready for review'],
    ['Resolved',rows.filter(t=>t.status==='Resolved').length,'Successfully completed']
  ];
  $('metrics').innerHTML = stats.map(([label,count,hint])=>'<section class="metric"><i>•</i><span>'+label+'</span><strong>'+count+'</strong><small>'+hint+'</small></section>').join('');
  if (page === 'Team members') return renderTeam();
  if (page === 'Performance') return renderPerformance();
  $('content').innerHTML = '<div class="layout"><section class="panel"><div class="panel-head"><h2>'+({employee:'My tickets',manager:'Incoming ticket queue',member:'My assigned tickets'}[role])+'</h2><span class="helper">'+rows.length+' tickets</span></div><div class="filters"><input id="search" aria-label="Search tickets" placeholder="Search by ticket ID or subject…" value="'+escapeHtml(search)+'"><select id="filter" aria-label="Filter by status">'+['All statuses','Open','In Progress','Awaiting Confirmation','Resolved'].map(s=>'<option '+(s===filter?'selected':'')+'>'+s+'</option>').join('')+'</select></div><div id="table" class="table-scroll"></div></section><div class="rail"><section class="panel callout"><h3>'+({employee:'A fix ready for your review',manager:'Critical tickets need you',member:'Keep employees in the loop'}[role])+'</h3><p>'+({employee:'Review submitted resolutions and confirm when everything works. You can reopen a ticket if the issue persists.',manager:'P1 tickets stay with the IT Manager. Assign other priorities to a team member who supports the category.',member:'Share troubleshooting updates in the conversation, then submit a clear resolution when the issue is fixed.'}[role])+'</p></section><section class="panel"><h3>IT support team</h3>'+team.map(m=>'<div class="person"><span class="avatar">'+m.name.slice(0,2).toUpperCase()+'</span><div><b>'+m.name+'</b><small>'+escapeHtml(m.categories.slice(0,2).join(' · '))+'</small></div></div>').join('')+'</section></div></div>';
  renderTable();
  $('search').oninput = e => {search=e.target.value;renderTable();};
  $('filter').onchange = e => {filter=e.target.value;renderTable();};
}
function renderTable() {
  let rows = visibleTickets().filter(t=>(t.id+' '+t.title).toLowerCase().includes(search.toLowerCase()) && (filter==='All statuses'||t.status===filter));
  if(page==='In progress') rows=rows.filter(t=>t.status==='In Progress');
  if(page==='Completed') rows=rows.filter(t=>t.status==='Resolved');
  $('table').innerHTML = rows.length ? '<table><thead><tr><th>Ticket / subject</th><th>Priority</th><th>Status</th><th>Assigned to</th></tr></thead><tbody>'+rows.map(t=>'<tr><td><button class="ticket-link" data-ticket="'+t.id+'"><small>'+t.id+' · '+t.type+'</small><b>'+escapeHtml(t.title)+'</b></button><small>'+escapeHtml(t.category)+'</small></td><td>'+badge(t.priority)+'</td><td>'+badge(t.status)+'</td><td>'+escapeHtml(t.assignee||'Unassigned')+'</td></tr>').join('')+'</tbody></table>' : '<div class="empty">No tickets match this view.</div>';
}
function openModal(html) {
  $('modal-content').innerHTML = html;
  if(!$('modal').open) $('modal').showModal();
}
const heading = title => '<div class="dialog-head"><h2>'+escapeHtml(title)+'</h2><button class="close" data-close aria-label="Close dialog">×</button></div>';
function createTicket() {
  openModal(heading('Create a ticket')+'<form id="create-form"><div class="row"><div><label for="type">Ticket type</label><select id="type" name="type"><option>Incident</option><option>Service Request</option></select></div><div><label for="priority">Priority</label><select id="priority" name="priority"><option value="P3">P3 · Medium</option><option value="P1">P1 · Critical</option><option value="P2">P2 · High</option><option value="P4">P4 · Low</option></select></div></div><label for="category">Category</label><select id="category" name="category"></select><div id="custom-wrap" hidden><label for="custom">Specific concern</label><input id="custom" name="custom"></div><label for="subject">Subject</label><input id="subject" name="subject" maxlength="160" required placeholder="A short summary of your request"><label for="description">Description</label><textarea id="description" name="description" maxlength="1000" required placeholder="Tell us what happened and how it affects your work"></textarea><div class="helper" id="counter">0 / 1,000 characters</div><label for="image">Supporting image</label><input id="image" type="file" accept="image/png,image/jpeg,image/webp"><p class="helper">Demo preview supports PNG, JPEG and WebP, up to 10 MB.</p><div id="image-preview"></div><div class="actions"><button type="button" data-close>Cancel</button><button class="primary">Create ticket</button></div></form>');
  let image = null;
  const custom = () => {
    const needed = $('type').value==='Incident' && $('category').value==='Others';
    $('custom-wrap').hidden=!needed; $('custom').required=needed;
  };
  const updateCategories=()=>{$('category').innerHTML=categories[$('type').value].map(c=>'<option>'+c+'</option>').join('');custom();};
  $('type').onchange=updateCategories; $('category').onchange=custom; updateCategories();
  $('description').oninput=()=>{$('counter').textContent=Array.from($('description').value).length+' / 1,000 characters';};
  $('image').onchange=()=>{
    const file=$('image').files[0]; image=null; $('image-preview').innerHTML='';
    if(!file)return;
    if(file.size>10*1024*1024 || !['image/png','image/jpeg','image/webp'].includes(file.type)){ $('image').value='';notice('Choose a supported image up to 10 MB.');return; }
    image=URL.createObjectURL(file);
    $('image-preview').innerHTML='<img class="preview" alt="Selected attachment" src="'+image+'">';
  };
  $('create-form').onsubmit=e=>{
    e.preventDefault();
    const data=new FormData(e.target);
    if(!data.get('subject').trim()||!data.get('description').trim()||($('custom').required&&!data.get('custom').trim()))return notice('Please fill in the required text.');
    const t={id:(data.get('type')==='Incident'?'INC-':'SR-')+(1046+tickets.length),title:data.get('subject').trim(),description:data.get('description').trim(),type:data.get('type'),category:data.get('category'),priority:data.get('priority'),owner:names.employee,assignee:'',status:'Open',custom:data.get('custom').trim(),image,messages:[],history:['Ticket created']};
    tickets.unshift(t); $('modal').close();render();notice(t.id+' created');showTicket(t.id);
  };
}
function showTicket(id) {
  selected=tickets.find(t=>t.id===id);
  const t=selected;
  if(!visibleTickets().includes(t))return;
  const owner=role==='employee'&&t.owner===names.employee;
  const handler=t.assignee===names[role]&&role!=='employee';
  const eligible=team.filter(m=>m.active&&m.categories.includes(t.category));
  openModal(heading(t.id+' · '+t.title)+'<p>'+badge(t.priority)+' '+badge(t.status)+'</p><p class="helper">'+escapeHtml(t.owner)+' · '+escapeHtml(t.category)+' · '+escapeHtml(t.assignee||'Awaiting assignment')+'</p><p>'+escapeHtml(t.description)+'</p>'+(t.custom?'<p>'+escapeHtml(t.custom)+'</p>':'')+(t.image?'<img class="preview" src="'+t.image+'" alt="Ticket attachment">':'')+
  (role==='manager'&&t.status==='Open'&&!t.assignee?'<form id="assignment"><h3>Review and assign</h3>'+(t.priority==='P1'?'<p>You will personally handle this critical ticket.</p>':'<label for="assignee">Eligible team member</label><select id="assignee" required><option value="">Select a member</option>'+eligible.map(m=>'<option>'+m.name+'</option>').join('')+'</select>')+'<div class="actions"><button class="primary">'+(t.priority==='P1'?'Handle this ticket':'Assign ticket')+'</button></div></form>':'')+
  (handler&&t.status==='Open'?'<button class="primary" id="start">Start work</button>':'')+
  (t.resolution?'<section class="conversation"><h3>Resolution</h3><p>'+escapeHtml(t.resolution)+'</p>'+(t.root?'<b>Root cause</b><p>'+escapeHtml(t.root)+'</p>':'')+'</section>':'')+
  (owner&&['Awaiting Confirmation','Resolved'].includes(t.status)?'<div class="actions">'+(t.status==='Awaiting Confirmation'?'<button class="primary" id="confirm">Confirm resolution</button>':'')+'<button id="reopen">Reopen ticket</button></div><form id="reopen-form" hidden><label for="reason">Why does this need more work?</label><textarea id="reason" maxlength="4000" required></textarea><button class="primary">Send for manager review</button></form>':'')+
  '<h3>Conversation</h3><div class="conversation">'+(t.messages.length?t.messages.map(m=>'<div class="message"><b>'+escapeHtml(m.author)+'</b>'+escapeHtml(m.text)+'</div>').join(''):'<p class="helper">No messages yet.</p>')+'</div>'+
  ((owner||handler)&&t.assignee&&t.status!=='Resolved'?'<form id="chat"><label for="message">Message</label><textarea id="message" required placeholder="Share an update or ask a question"></textarea><div class="actions"><button>Send message</button></div></form>':'')+
  (handler&&t.status==='In Progress'?'<form id="resolve"><h3>Submit resolution</h3><label for="resolution">What did you do to fix it?</label><textarea id="resolution" maxlength="8000" required></textarea>'+(t.type==='Incident'?'<label for="root">Root cause</label><textarea id="root" maxlength="4000" required></textarea>':'')+'<div class="actions"><button class="primary">Send for employee confirmation</button></div></form>':'')+
  '<h3>Activity history</h3><ol class="history">'+t.history.map(h=>'<li>'+escapeHtml(h)+'</li>').join('')+'</ol>');
  const refresh=message=>{render();showTicket(t.id);notice(message);};
  if($('assignment'))$('assignment').onsubmit=e=>{e.preventDefault();t.assignee=t.priority==='P1'?names.manager:$('assignee').value;t.history.push('Reviewed and assigned to '+t.assignee);refresh('Assignment saved');};
  if($('start'))$('start').onclick=()=>{t.status='In Progress';t.history.push(t.assignee+' started investigation');refresh('Work started');};
  if($('chat'))$('chat').onsubmit=e=>{e.preventDefault();const text=$('message').value.trim();if(!text)return; t.messages.push({author:names[role],text});refresh('Message sent');};
  if($('resolve'))$('resolve').onsubmit=e=>{e.preventDefault();const resolution=$('resolution').value.trim();const root=$('root')?.value.trim();if(!resolution||(t.type==='Incident'&&!root))return notice('Enter the resolution and required root cause.');t.resolution=resolution;t.root=root;t.status='Awaiting Confirmation';t.history.push('Resolution submitted by '+t.assignee+': '+resolution+(root?' · Root cause: '+root:''));refresh('Sent for employee confirmation');};
  if($('confirm'))$('confirm').onclick=()=>{t.status='Resolved';t.history.push('Resolution confirmed by '+t.owner);refresh('Ticket resolved');};
  if($('reopen'))$('reopen').onclick=()=>{$('reopen-form').hidden=false;$('reason').focus();};
  if($('reopen-form'))$('reopen-form').onsubmit=e=>{e.preventDefault();const reason=$('reason').value.trim();if(!reason)return; t.history.push('Reopened by '+t.owner+': '+reason);t.status='Open';t.assignee='';t.resolution='';t.root='';refresh('Returned to manager review');};
}
function renderTeam() {
  $('content').innerHTML='<div class="team-grid">'+team.map((m,i)=>'<section class="panel"><span class="avatar">'+m.name.slice(0,2).toUpperCase()+'</span><h2>'+m.name+'</h2><p class="helper">'+escapeHtml(m.categories.join(' · '))+'</p><p>'+tickets.filter(t=>t.assignee===m.name&&t.status!=='Resolved').length+' active tickets</p><button data-member="'+i+'">'+(m.active?'Deactivate':'Activate')+'</button></section>').join('')+'</div><p class="helper">Inactive members are excluded from new assignments in this demo.</p>';
}
function renderPerformance() {
  const total=tickets.length;
  $('content').innerHTML='<section class="panel" style="padding:26px;max-width:750px"><h2>Ticket status breakdown</h2><p class="helper">Calculated from the current demo tickets</p>'+['Open','In Progress','Awaiting Confirmation','Resolved'].map(s=>{const n=tickets.filter(t=>t.status===s).length;return '<div>'+s+' · '+n+' tickets</div><div class="bar"><span style="width:'+n/total*100+'%"></span></div>';}).join('')+'</section>';
}
$('role').onchange=e=>{role=e.target.value;page='Dashboard';search='';filter='All statuses';$('modal').close();render();};
$('new-ticket').onclick=createTicket;
document.addEventListener('click',e=>{
  const close=e.target.closest('[data-close]');if(close)return $('modal').close();
  const nav=e.target.closest('[data-page]');if(nav){if(nav.dataset.page==='Create ticket')return createTicket();page=nav.dataset.page;render();}
  const ticket=e.target.closest('[data-ticket]');if(ticket)showTicket(ticket.dataset.ticket);
  const member=e.target.closest('[data-member]');if(member){team[member.dataset.member].active=!team[member.dataset.member].active;renderTeam();}
});
render();
